use eyre::Result;
use image::RgbImage;
use openh264::{decoder::Decoder, formats::YUVSource};
use std::collections::VecDeque;
use tokio::{net::UdpSocket, sync::watch};

#[allow(unused_imports)]
use tracing::{debug, info, trace, warn};

const NAL_MIN_0_COUNT: usize = 2;

/// Extracts the starting position of the n-th NAL unit from the given byte array.
///
/// Copied and adapted from openh264 library.
#[inline]
fn nth_nal_index(buffer: &VecDeque<u8>, nth: usize) -> Option<usize> {
    let mut count_0 = 0;
    let mut n = 0;

    for (i, byte) in buffer.iter().enumerate() {
        match byte {
            0 => count_0 += 1,
            1 if count_0 >= NAL_MIN_0_COUNT => {
                if n == nth {
                    return Some(i - NAL_MIN_0_COUNT);
                } else {
                    count_0 = 0;
                    n += 1;
                }
            }
            _ => count_0 = 0,
        }
    }

    None
}

/// Produces all complete h264 NAL units from the given buffer, leaving
/// any remaining data in the buffer.
///
/// Copied from openh264 library.
fn nal_units(buffer: &mut VecDeque<u8>) -> impl Iterator<Item = Vec<u8>> + '_ {
    std::iter::from_fn(move || {
        let first = nth_nal_index(buffer, 0);
        let next = nth_nal_index(buffer, 1);
        match (first, next) {
            (Some(_), Some(n)) => {
                // Note that we're taking a copy of the bytes here, likely in a somewhat inefficient
                // manner. The alternative is to compact the VecDeque and return a &[u8].
                // Would need to micro-benchmark in order to figure out which one is more efficient.
                let nal: Vec<_> = buffer.drain(0..n).collect();
                Some(nal)
            }
            _ => None,
        }
    })
}

/// Receives video frames from the drone's UDP connection, and writes them into [sink].
///
/// Blocks until [sink] is closed. Assumes a "raw" annex-b h264 byte stream sent over UDP. Yep,
/// this is in the official Tello format.
pub async fn watch_latest_frame(sink: watch::Sender<RgbImage>, socket: UdpSocket) -> Result<()> {
    let mut decoder = Decoder::new()?;

    let mut rgb_buffer = vec![0u8; 2000 * 2000 * 3]; // upper bound for image size
    let mut packet_buffer = vec![0u8; 2000]; // holds on UDP packet at a time
    let mut h264_buffer = VecDeque::new(); // ringbuffer holding the h264 byte stream

    loop {
        // Write next packet into h264 byte stream buffer
        trace!("await h264 packet");
        let size = socket.recv(packet_buffer.as_mut()).await?;
        trace!("got h264 packet");
        h264_buffer.extend(&packet_buffer[0..size]);

        // Extract NAL units and decode them one by one into RGB frames.
        for packet in nal_units(&mut h264_buffer) {
            trace!("walking nal unit");
            match decoder.decode(packet.as_slice()) {
                Ok(Some(frame)) => {
                    trace!("got frame");

                    let num_bytes = (frame.width() * frame.height() * 3) as usize;
                    if num_bytes > rgb_buffer.len() {
                        warn!(
                            "Frame size exceeded buffer size ({} bytes)",
                            rgb_buffer.len()
                        );
                    } else {
                        let sized_rgb_buffer = rgb_buffer[0..num_bytes].as_mut();
                        frame.write_rgb8(sized_rgb_buffer);
                        let image = RgbImage::from_vec(
                            frame.width() as u32,
                            frame.height() as u32,
                            sized_rgb_buffer.to_vec(),
                        )
                        .expect("Size mismatch; this is a bug");

                        trace!("updated frame");

                        if sink.send(image).is_err() {
                            warn!("exiting as there are no receivers");
                            return Ok(());
                        }
                    }
                }
                Ok(None) => trace!("skipping empty NAL unit"),
                Err(e) => {
                    trace!(%e, "skipping packet h264 decoder is unhappy with")
                }
            }
        }
    }
}
