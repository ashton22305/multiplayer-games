# Deployment

This document covers deploying the Vue SPA frontend. The Rust game-server, API, and database land
in later phases and are not covered here yet.

## How it works

- **CI** (`.github/workflows/ci.yml`) runs on pull requests and is reusable (`workflow_call`):
  typecheck, tests (Vitest), and a production build.
- **CD** is a per-branch pipeline that runs CI first, then deploys only if CI passed:
  - `deploy-web-dev.yml` — on push to `main`: runs CI, then the `deploy` job (`needs: ci`) targets
    the `dev` environment.
  - `deploy-web-prod.yml` — on push to `prod`: runs CI, then deploys to the `prod` environment.
  - Both delegate the actual deploy to the reusable `deploy-web.yml`, which builds the SPA and
    `rsync`s `web/dist/` to the VM over SSH. Caddy serves each environment's subdomain with SPA
    history-mode fallback.
- The deploy job is gated on the repository variable `DEPLOY_ENABLED`. Until it equals `true`, the
  deploy job is **skipped** (it does not run and does not report a false success). Set
  `DEPLOY_ENABLED=true` once the VM, DNS, and environment secrets below are in place.

`main` is the development line; promote a release by pushing/merging `main` into `prod`.

## What still needs to be configured

### 1. VM (Oracle Cloud Always Free ARM, or similar)
- Provision the VM and install Caddy.
- Create the web roots and make them writable by the deploy user:
  - `dev`: `/var/www/arcade-dev`
  - `prod`: `/var/www/arcade-prod`

### 2. Deploy SSH access
- Generate an SSH keypair for CI (`ssh-keygen -t ed25519`).
- Add the public key to the deploy user's `~/.ssh/authorized_keys` on the VM.
- The private key goes into the GitHub Environment secrets below.

### 3. DNS
- Point both subdomains at the VM's public IP (A / AAAA records):
  - `dev.<your-domain>` (development)
  - `<your-domain>` (production)
- The GitHub Student Pack includes a free domain (Namecheap) for a year.

### 4. Caddy
- Use `deploy/Caddyfile` as a starting point; replace `dev.example.com` / `example.com` with your
  real subdomains. Caddy obtains and renews TLS automatically.
- Reload after editing: `caddy reload --config /etc/caddy/Caddyfile`.

### 5. GitHub branches, environments, and secrets
- Create the `main` and `prod` branches.
- Create two GitHub Environments: `dev` and `prod` (Settings → Environments). Consider a required
  reviewer / branch protection on `prod`.
- In **each** environment, set these secrets:
  - `DEPLOY_HOST` — VM hostname or IP
  - `DEPLOY_USER` — deploy SSH user
  - `DEPLOY_SSH_KEY` — the private key from step 2
  - `DEPLOY_WEB_PATH` — web root for that env (`/var/www/arcade-dev` or `/var/www/arcade-prod`)
- In each environment, set this variable:
  - `SITE_URL` — the environment's URL (e.g. `https://dev.<your-domain>`), shown on the deployment.
- Set the repository variable `DEPLOY_ENABLED` to `true` to turn deploys on (Settings → Secrets and
  variables → Actions → Variables). While unset, deploy jobs are skipped.

Once configured: push to `main` deploys dev; push to `prod` deploys prod.

## Local development

```sh
nvm use            # uses web/.nvmrc (Node 22)
npm install
npm run dev        # Vite dev server
npm run typecheck
npm test
npm run build      # outputs web/dist
```

> WSL note: run Node from the Linux side (e.g. via nvm). The Windows Node binary cannot install
> native dependencies (esbuild) on the WSL filesystem.
