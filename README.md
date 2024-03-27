<!-- markdownlint-disable-next-line -->
<div align="center">

<img src="./assets/banner.png" onerror="this.style.display='none'" />

<br/>

# Rust Nation Hackathon 2024

**An embedded computer vision hackathon**

[![Helsing](https://img.shields.io/badge/helsing-hosted-black.svg)](https://helsing.ai)
[![Rust](https://img.shields.io/static/v1?message=nation&color=000000&logo=Rust&logoColor=FFFFFF&label=rust)](https://rustnationuk.com)

</div>

## Welcome

Welcome to Helsings Rust Nation Hackathon! This is the very first edition of
our hackathon and we are glad to have you onboard. Enjoy!

## Rules & Setup

- You are going to work in **teams of 3-4** people.
- You may not move the drone outside of the enclosure
- Connect to the `hs-rust-nation` wifi

## Challenge

<div align="center">
    <img src="./assets/top.png" width="512" onerror="this.style.display='none'" />
    <br>
    <em>A topdown-view of the challenge setup</em>
</div>

<br>

You and your team are resposible for writing an application that makes use of
computer vision, distributed systems and networking to maneouver your teams car
(identified by the colored LED, in the example setup above: green) onto a color
coded target (in the example setup above: blue).

The challenge you need to solve here arises from the fact that *the car itself
has no sensors* and all information you have is coming from the drones camera
and your computer vision algorithm.

<div align="center">
    <img src="./assets/side.png" width="512" onerror="this.style.display='none'" />
    <br>
    <em>A side-view of the challenge setup</em>
</div>

<br>

The car is equipped with a raspberry pi that has access to the camera stream of
the drone and the cars hardware (e.g. you can drive the car from the raspberry
pi).

Given the above setup and the libraries provided by helsing you should write
the highlevel application logic to:

- Identify your cars position
- Identify the orientation of the car
- Move the car into the color coded target area

> Please Note: You are expected to develop this on your own laptop. Helsing
> provides you libraries and deployment tooling, aswell as the drone and the
> car, to make the above achievable in 60-90 minutes.

## Quickstart

**Please make use of the
[template](https://github.com/helsing-ai/rust-nation-starter) in order to have
a seamless deployment experience**

- Form a team of 3-4 people and obtain your teams hardware from helsing staff.
- Fork the repository and grant your team members access
- Clone your teams repository and do a test deployment (to verify everythign works)
- Start hacking!

## Deploying

```
$ rsync --exclude target -r . hack@<team>:/solution
$ ssh hack@team
```

## Positioning the drone

### Installing `aviator`

```
$ ssh hack@team "cargo instal --git github.com/helsing-ai/rust-nation --bin aviator
$ aviator
```

### Using `aviator`

To use aviator you should run it at all times at is takes care of managing your drone!

You need to position the drone yourself (judging the FOV).

You can open `http://<team>:3000/camera` to see the drones image and use
`./scripts/aviate <team> <command>` to position it manually.
