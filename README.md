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
- Connect to the `hs-rust-nation` wifi - please ask for the password from one of the team.

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

The car is equipped with a Raspberry Pi that has access to the camera stream of
the drone and the cars hardware (e.g. you can drive the car from the raspberry
pi).

Given the above setup and the libraries provided by helsing you should write
the highlevel application logic to:

- Identify your car's position
- Identify the orientation of the car
- Move the car into the color coded target area

> Please note: You are expected to develop this on your own laptop. Helsing
> provides you libraries and deployment tooling, as well as the drone and the
> car, to make the above achievable in 1-2 hours.

## Quickstart

**Please make use of the
[template](https://github.com/helsing-ai/rust-nation-starter) in order to have
a seamless deployment experience**

- Form a team of 3-4 people and obtain your team's hardware from helsing staff.
- Fork the repository and grant your team members access
- Clone your teams repository and do a test deployment (to verify everythign works)
- Start hacking!
- When you're ready to start driving the car, please ask and we'll connect the power supply.

## Deploying

```
$ rsync --exclude target -r ./ hack@<team>:/home/hack/
$ ssh hack@<team>
```

> Having DNS issues? Try using `nmap` and use the car's IP directly. (`nmap -sP 192.168.50.0/24`)

## Positioning the drone

You need to position the drone yourself using `./scripts/aviate` (judging the FOV).

You can open `http://<car-name>:3000/camera` to see the drones image and use
`./scripts/aviate <car-name> <command>` to position it manually.

## FAQ / Trubleshooting

### What is our team name?

Your team name is determined by the drones label!

### What is the ssh / sudo password?

A helsing staff member will provide you with your credentials

### I can't ssh into the car..?

- Verify that the cars raspberry pi is on
- You are on the `hs-rust-nation` network
- Check that you can reach the car by `ping <team>`
- If that doesnt work, try `nmap -sP 192.168.50.0/24` or ask helsing staff

### I don't have access to the aviator (`http://<car-name>:3000`)..?

- Check that the drone is on (touch the button on the side once)
- Check that you have a `wlp1s0u1u2` interfance `ifconfig`
- Wait until you get a ip from the drone on that interface using `watch ifconfig`
- If that doesnt happen try `sudo systemctl restart drone-wifi`
- Once you have the ip do a `sudo systemctl restart aviator`
