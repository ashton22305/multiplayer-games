# Deployment

This document covers deploying the Vue SPA frontend to **Cloudflare Pages**. The Rust game-server,
API, and database land in later phases and are not covered here yet.

## How it works

- **CI** (`.github/workflows/ci.yml`) runs on pull requests and is reusable (`workflow_call`):
  typecheck, tests (Vitest), and a production build.
- **CD** is a per-branch pipeline that runs CI first, then deploys only if CI passed:
  - `deploy-web-dev.yml` — on push to `main`: runs CI, then the `deploy` job (`needs: ci`) deploys
    to the `dev` environment.
  - `deploy-web-prod.yml` — on push to `prod`: runs CI, then deploys to the `prod` environment.
  - Both delegate to the reusable `deploy-web.yml`, which builds the SPA and runs
    `wrangler pages deploy` (via `cloudflare/wrangler-action`). The Pages branch is the triggering
    git branch (`github.ref_name`), so Cloudflare treats `prod` as the production deployment and
    `main` as a preview deployment.
- The deploy job is gated on the repository variable `DEPLOY_ENABLED`. Until it equals `true`, the
  deploy job is **skipped** (it does not run and does not report a false success). Set
  `DEPLOY_ENABLED=true` once the Cloudflare project and secrets below are in place.

`main` is the development line (preview); promote a release by pushing/merging `main` into `prod`.

## Environments and URLs

- **dev** (`main` branch) → `main.<project>.pages.dev`, or `dev.<your-domain>` if you attach a
  branch custom domain.
- **prod** (`prod` branch) → `<project>.pages.dev` and your production custom domain.

## What needs to be configured

### 1. Cloudflare Pages project
- Create a free Cloudflare account, then a Pages project (Dashboard → Workers & Pages → Create →
  Pages). Set the **production branch** to `prod`.
- No GitHub Pro or public repo required — the workflow uploads the built `dist`, independent of repo
  visibility.

### 2. GitHub repository secrets
- `CLOUDFLARE_API_TOKEN` — a token with the "Cloudflare Pages: Edit" permission.
- `CLOUDFLARE_ACCOUNT_ID` — your Cloudflare account ID.

### 3. GitHub variables
- `CF_PAGES_PROJECT` (repository variable) — the Pages project name.
- `DEPLOY_ENABLED` (repository variable) — set to `true` to enable deploys. While unset, deploy
  jobs are skipped.
- `SITE_URL` (per-environment variable, optional) — the dev/prod URL shown on the GitHub deployment.

### 4. GitHub branches and environments
- Create the `main` and `prod` branches.
- Create two GitHub Environments: `dev` and `prod` (Settings → Environments). Consider a required
  reviewer / branch protection on `prod`.

### 5. Custom domains (optional)
- **prod:** add your domain (e.g. `yourdomain.com` or `play.yourdomain.com`) to the production
  deployment (Pages project → Custom domains). Cloudflare provisions HTTPS automatically.
- **dev:** attach `dev.yourdomain.com` to the `main` branch (Pages → add a custom domain to a
  branch). This only works when the domain's DNS is on Cloudflare with a **proxied** record; with
  external DNS the alias resolves to production instead.
- The GitHub Student Pack includes a free domain (Namecheap) for a year.

Once configured: push to `main` deploys dev (preview); push to `prod` deploys prod (production).

## SPA routing

`web/public/_redirects` contains `/*  /index.html  200` so client-side (history-mode) routes such
as `/play/snake` resolve correctly on direct load or refresh.

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
