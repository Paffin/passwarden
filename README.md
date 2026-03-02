# Passwarden

**A Vaultwarden fork with extended features for self-hosted password management.**

Based on [Vaultwarden](https://github.com/dani-garcia/vaultwarden) — an alternative server implementation of the Bitwarden Client API, written in Rust and compatible with [official Bitwarden clients](https://bitwarden.com/download/).

---

[![GitHub Release](https://img.shields.io/github/release/Paffin/passwarden.svg?style=for-the-badge&color=005AA4)](https://github.com/Paffin/passwarden/releases/latest)
[![AGPL-3.0 Licensed](https://img.shields.io/github/license/Paffin/passwarden.svg?style=for-the-badge&color=944000)](https://github.com/Paffin/passwarden/blob/main/LICENSE.txt)
[![Issues](https://img.shields.io/github/issues/Paffin/passwarden.svg?style=for-the-badge&color=005AA4)](https://github.com/Paffin/passwarden/issues)

> [!NOTE]
> Passwarden is an independent fork of Vaultwarden. Please report bugs and suggestions to [our issue tracker](https://github.com/Paffin/passwarden/issues), not to the upstream Vaultwarden or official Bitwarden channels.

## What's different from Vaultwarden?

Passwarden extends Vaultwarden with features that the upstream project hasn't implemented yet:

### Implemented

- **Organization Sends** — Send items can now belong to organizations, with proper revision tracking for all org members
- **MySQL/PostgreSQL Backup** — Database backup from admin panel now supports all database backends (via `mysqldump`/`pg_dump`)

### Roadmap

| Priority | Feature | Status |
|----------|---------|--------|
| P0 | New Device Login Protection | Planned |
| P0 | Passwordless Login via Passkeys | Planned |
| P1 | Custom User Roles (replacing HACK workarounds) | Planned |
| P1 | SCIM 2.0 Provisioning | Planned |
| P1 | Tags system for vault items | Planned |
| ~~P1~~ | ~~MySQL/PostgreSQL backup support~~ | Done |
| P2 | Webhook/Event Delivery API | Planned |
| P2 | P2P Password Sharing | Planned |
| P2 | Mobile-Responsive Admin UI | Planned |

## Upstream Features

All features from Vaultwarden are included:

 * [Personal Vault](https://bitwarden.com/help/managing-items/)
 * [Send](https://bitwarden.com/help/about-send/)
 * [Attachments](https://bitwarden.com/help/attachments/)
 * [Website icons](https://bitwarden.com/help/website-icons/)
 * [Personal API Key](https://bitwarden.com/help/personal-api-key/)
 * [Organizations](https://bitwarden.com/help/getting-started-organizations/)
   - [Collections](https://bitwarden.com/help/about-collections/),
     [Password Sharing](https://bitwarden.com/help/sharing/),
     [Member Roles](https://bitwarden.com/help/user-types-access-control/),
     [Groups](https://bitwarden.com/help/about-groups/),
     [Event Logs](https://bitwarden.com/help/event-logs/),
     [Admin Password Reset](https://bitwarden.com/help/admin-reset/),
     [Directory Connector](https://bitwarden.com/help/directory-sync/),
     [Policies](https://bitwarden.com/help/policies/)
 * [Multi/Two Factor Authentication](https://bitwarden.com/help/bitwarden-field-guide-two-step-login/)
   - [Authenticator](https://bitwarden.com/help/setup-two-step-login-authenticator/),
     [Email](https://bitwarden.com/help/setup-two-step-login-email/),
     [FIDO2 WebAuthn](https://bitwarden.com/help/setup-two-step-login-fido/),
     [YubiKey](https://bitwarden.com/help/setup-two-step-login-yubikey/),
     [Duo](https://bitwarden.com/help/setup-two-step-login-duo/)
 * [Emergency Access](https://bitwarden.com/help/emergency-access/)
 * [Admin Backend](https://github.com/dani-garcia/vaultwarden/wiki/Enabling-admin-page)

## Usage

> [!IMPORTANT]
> The web-vault requires a secure context for the [Web Crypto API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Crypto_API).
> It will only work via `http://localhost:8000` or with [HTTPS enabled](https://github.com/dani-garcia/vaultwarden/wiki/Enabling-HTTPS).

### Docker Compose

```yaml
services:
  passwarden:
    image: vaultwarden/server:latest  # or build from this repo
    container_name: passwarden
    restart: unless-stopped
    environment:
      DOMAIN: "https://pw.domain.tld"
    volumes:
      - ./pw-data/:/data/
    ports:
      - 127.0.0.1:8000:80
```

### Building from source

```shell
# Clone the repository
git clone https://github.com/Paffin/passwarden.git
cd passwarden

# Build with SQLite support
cargo build --features sqlite --release

# Build with MySQL support
cargo build --features mysql --release

# Build with PostgreSQL support
cargo build --features postgresql --release
```

For more detailed build instructions, see the [Vaultwarden Wiki](https://github.com/dani-garcia/vaultwarden/wiki/Building-binary).

## Syncing with upstream

Passwarden regularly syncs with the latest Vaultwarden releases to incorporate upstream fixes and improvements.

```shell
git remote add upstream https://github.com/dani-garcia/vaultwarden.git
git fetch upstream
git merge upstream/main
```

## License

This project is licensed under the [GNU Affero General Public License v3.0](LICENSE.txt), same as the upstream Vaultwarden project.

## Acknowledgments

- [Vaultwarden](https://github.com/dani-garcia/vaultwarden) by Daniel Garcia and contributors
- [Bitwarden](https://bitwarden.com/) for the client applications and API specification

**This project is not associated with [Bitwarden](https://bitwarden.com/) or Bitwarden, Inc.**
