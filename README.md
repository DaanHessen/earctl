<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project_license][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<br />
<div align="center">
  <a href="https://github.com/DaanHessen/earctl">
    <img src="docs/ear-web/res/icons/256x256.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">Ear Control API</h3>

  <p align="center">
    A Rust API/CLI that mirrors the Nothing ear-web backend so every device on your network can control your earbuds.
    <br />
    <a href="#usage"><strong>Jump to usage »</strong></a>
    <br />
    <br />
    <a href="https://github.com/DaanHessen/earctl/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/DaanHessen/earctl/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#cli-reference">CLI Reference</a></li>
    <li><a href="#api-endpoints">API Endpoints</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

## About The Project

[![Product Name Screen Shot][product-screenshot]](https://github.com/DaanHessen/earctl)

earctl is a Rust rewrite of the backend logic used by the community ear-web tool. It exposes the same Bluetooth Serial Port Profile commands through a REST API and a small CLI. That means you can automate ANC, EQ, latency mode, gestures, and the rest of the Nothing/CMF settings without launching a browser. The server stays up in the background and automatically binds to your earbuds’ RFCOMM port whenever it sees them online.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Built With

* [![Rust][Rust-img]][Rust-url]
* [![Tokio][Tokio-img]][Tokio-url]
* [![Axum][Axum-img]][Axum-url]
* [![BlueZ][BlueZ-img]][BlueZ-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Getting Started

Follow these steps to run the API locally.

### Prerequisites

* Rust toolchain (1.75+ recommended)
* `bluez`, `bluez-utils` (for `bluetoothctl`)
* `bluez-deprecated-tools` (provides `sdptool` for RFCOMM channel discovery)
* Your earbuds paired in the desktop Bluetooth UI

### Installation

1. Clone the repo
   ```sh
   git clone https://github.com/DaanHessen/earctl.git
   cd earctl
   ```
2. Build in dev mode
   ```sh
   cargo build
   ```
3. Start the API server (it defaults to `127.0.0.1:8787`)
   ```sh
   earctl server --addr 0.0.0.0:8787
   ```
4. Keep your earbuds connected through the system tray and run the auto connect helper (details below).

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Usage

1. Ensure the server is running (`earctl server --addr 0.0.0.0:8787`) and your earbuds are connected via the OS Bluetooth menu.
2. Auto-connect from any terminal:
   ```sh
   earctl auto-connect --name "Nothing Ear"
   ```
   The server queries BlueZ for connected devices, discovers the Nothing RFCOMM channel via `sdptool`, and opens the session. Supply `--channel 15` if you need to override the detected value.
3. Toggle ANC, EQ, latency, gestures, LED color, or trigger Find My Buds either via the REST endpoints or the CLI.
4. Your top bar extension (or any other app) can poll `/api/session` to see when the buds are available.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## CLI Reference

All commands accept `--endpoint` to target a different API host. Defaults to `http://127.0.0.1:8787`.

| Command | Description |
| --- | --- |
| `earctl server --addr 0.0.0.0:8787` | Boot the HTTP API. |
| `earctl connect --address 3C:B0:ED:C4:B0:31 --channel 15` | Manually open a session when you already know the MAC + RFCOMM channel. |
| `earctl auto-connect --name "Nothing Ear"` | Locate the connected buds, detect their RFCOMM channel, and open the session (optionally add `--channel` to override). |
| `earctl session` | Show session metadata (model, port, UUID). |
| `earctl detect` | Read serial/SKU from the earbuds over SPP. |
| `earctl battery` | Print battery readings for left/right/case. |
| `earctl anc get` / `earctl anc set transparency` | Read or change ANC mode (off / transparency / ANC strengths). |
| `earctl eq get` / `earctl eq set --mode 3` | Get or switch the preset EQ. |
| `earctl custom-eq get` / `earctl custom-eq set --bass 2 --mid 0 --treble -1` | Manage 3-band custom EQ values. |
| `earctl latency get` / `earctl latency set true|false` | Toggle low-latency gaming mode. |
| `earctl in-ear get` / `earctl in-ear set true|false` | Toggle in-ear detection (where supported). |
| `earctl enhanced-bass get` / `earctl enhanced-bass set --enabled true --level 4` | Manage enhanced bass toggle + level. |
| `earctl personalized-anc get` / `earctl personalized-anc set true|false` | Personalized ANC switch (Ear (2) & Nothing Ear). |
| `earctl ring --enable true --side left` | Trigger Find My Buds (side optional). |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## API Endpoints

| Method | Path | Body | Description |
| --- | --- | --- | --- |
| `POST` | `/api/session/auto-connect` | `{ address?, name?, channel?, sku? }` | Locate connected earbuds through `bluetoothctl`, discover their RFCOMM channel via SDP, and open a session. |
| `POST` | `/api/session/connect` | `{ address, channel, model? }` | Open a session using an explicit Bluetooth MAC + channel. |
| `GET` | `/api/session` | – | Return session UUID, port path, and detected model. |
| `DELETE` | `/api/session` | – | Drop the active session. |
| `POST` | `/api/session/detect` | – | Read serial/SKU for the connected device. |
| `POST` | `/api/session/model` | `{ model_id? , sku? , base? }` | Override the model metadata. |
| `GET/POST` | `/api/battery` | – / – | Read battery info (GET only). |
| `GET/POST` | `/api/anc` | `{ "level": "transparency" }` | Read or set ANC. |
| `GET/POST` | `/api/eq` | `{ "mode": 2 }` | Read or set the main preset. |
| `GET/POST` | `/api/eq/custom` | `{ "bass": 2.0, "mid": 0.0, "treble": -1.0 }` | Manage custom EQ bands. |
| `GET/POST` | `/api/enhanced-bass` | `{ "enabled": true, "level": 4 }` | Toggle enhanced bass (supported models). |
| `GET/POST` | `/api/personalized-anc` | `{ "enabled": true }` | Personalized ANC (Ear (2)/Nothing Ear). |
| `GET/POST` | `/api/in-ear` | `{ "detection_enabled": true }` | In-ear detection (not available on Ear (open)). |
| `GET/POST` | `/api/latency` | `{ "low_latency_enabled": true }` | Gaming/low-latency switch. |
| `GET` | `/api/firmware` | – | Firmware string from the buds. |
| `POST` | `/api/ear-fit` | – | Launch ear-fit test. |
| `GET` | `/api/ear-fit` | – | Read ear-fit result. |
| `GET/POST` | `/api/gestures` | `[{ device, common, gesture_type, action }]` | Read or set gesture mappings. |
| `GET/POST` | `/api/led-case` | `{ "pixels": [[r,g,b], ...] }` | Case LED colors (Ear (1)). |
| `POST` | `/api/ring` | `{ "enable": true, "side": "left" }` | Find My Buds tone. |

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Roadmap

- [ ] 

See the [open issues](https://github.com/DaanHessen/earctl/issues) for anything else that’s planned.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Contributing

Contributions are welcome: open an issue if you found a bug or send a pull request with the fix.

Top contributors:

<a href="https://github.com/DaanHessen/earctl/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=DaanHessen/earctl" alt="contrib.rocks image" />
</a>

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## License

Distributed under the GNU Affero GPL-3.0 license. See `LICENSE` for details.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Contact

Daan Hessen – daanh2002@gmail.com

Project Link: [https://github.com/DaanHessen/earctl](https://github.com/DaanHessen/earctl)

<p align="right">(<a href="#readme-top">back to top</a>)</p>

## Acknowledgments

* RapidZapper, Bendix, and the rest of the ear-web crew for the groundwork.
* Everyone reverse-engineering SPP payloads on Discord.
* The BlueZ maintainers.

<p align="right">(<a href="#readme-top">back to top</a>)</p>

[contributors-shield]: https://img.shields.io/github/contributors/DaanHessen/earctl.svg?style=for-the-badge
[contributors-url]: https://github.com/DaanHessen/earctl/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/DaanHessen/earctl.svg?style=for-the-badge
[forks-url]: https://github.com/DaanHessen/earctl/network/members
[stars-shield]: https://img.shields.io/github/stars/DaanHessen/earctl.svg?style=for-the-badge
[stars-url]: https://github.com/DaanHessen/earctl/stargazers
[issues-shield]: https://img.shields.io/github/issues/DaanHessen/earctl.svg?style=for-the-badge
[issues-url]: https://github.com/DaanHessen/earctl/issues
[license-shield]: https://img.shields.io/github/license/DaanHessen/earctl.svg?style=for-the-badge
[license-url]: https://github.com/DaanHessen/earctl/blob/master/LICENSE
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://www.linkedin.com/in/daan-hessen-552789236/
[product-screenshot]: docs/ear-web/res/MainControl/hero.png
[Rust-img]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[Rust-url]: https://www.rust-lang.org/
[Tokio-img]: https://img.shields.io/badge/Tokio-000000?style=for-the-badge&logo=rust&logoColor=white
[Tokio-url]: https://tokio.rs/
[Axum-img]: https://img.shields.io/badge/Axum-1E88E5?style=for-the-badge&logo=rust&logoColor=white
[Axum-url]: https://github.com/tokio-rs/axum
[BlueZ-img]: https://img.shields.io/badge/BlueZ-0A5CAB?style=for-the-badge&logo=bluetooth&logoColor=white
[BlueZ-url]: http://www.bluez.org/
