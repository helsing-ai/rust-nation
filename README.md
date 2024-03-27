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

You and your team are responsible for writing an application that makes use of
computer vision, distributed systems and networking to manoeuver your teams car
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

- Form a team of 3-4 people and obtain your teams hardware from helsing staff.
- Fork the repository and grant your team members access
- Clone your teams repository and do a test deployment (to verify everything works)
- Start hacking!

## Deploying

```
$ rsync --exclude target -r . hack@<team>:/solution
$ ssh hack@team
```

> Having DNS issues? Try using `nmap` and use the car's IP directly. (`nmap -sP 192.168.50.0/24`)

## Positioning the drone

You need to position the drone yourself using `./scripts/aviate` (judging the FOV).

You can open `http://<car-name>:3000/camera` to see the drones image and use
`./scripts/aviate <car-name> <command>` to position it manually.
