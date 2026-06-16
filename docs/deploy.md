# Deploy runbook — plotplot.ai

The site is a static page served by **Cloudflare Workers (Static Assets)**. No build step.

- Deploy root: `public/` (`index.html` + `plotplot-mark.svg`)
- Config: `wrangler.jsonc` (`assets.directory: ./public`, `account_id` set)
- Tooling: `wrangler` (devDependency); `npm run deploy` / `npm run dev` / `npm run check`

## Manual deploy

```bash
npm install      # first time
npm run deploy   # -> https://plotplot.<account>.workers.dev
```

`npm run dev` serves it locally (Cloudflare-accurate) at http://localhost:8787.

## Auto-deploy on merge (CI)

`.github/workflows/deploy.yml` runs `wrangler deploy` on every push to `main`/`master`
(and on manual "Run workflow"). One-time setup:

1. **Create a Cloudflare API token** — dashboard → My Profile → API Tokens → Create Token
   → template **"Edit Cloudflare Workers"**. Scope it to your account and include the
   `plotplot.ai` + `plotplot.io` zones (or All zones). This grants Workers Scripts: Edit,
   Workers Routes: Edit, Account: Read, Zone DNS: Edit — enough to deploy and attach domains.
2. **Add it as a repo secret** named `CLOUDFLARE_API_TOKEN`:
   ```bash
   gh secret set CLOUDFLARE_API_TOKEN   # paste the token when prompted
   ```
   (or GitHub → repo → Settings → Secrets and variables → Actions → New secret)
3. Merge to the default branch (or run the workflow manually) → it deploys.

## Custom domains (plotplot.ai + plotplot.io)

Both domains (apex + www) serve the same Worker via `routes` with `custom_domain: true`.

**Prerequisite: the zones must be Active.** Moving from GoDaddy means repointing the
nameservers at GoDaddy to the two Cloudflare nameservers shown on each zone's overview,
then waiting for propagation. Check:

```bash
dig NS plotplot.ai     # Active once this returns *.ns.cloudflare.com (not domaincontrol.com)
```

Once both zones are Active, uncomment the `routes` block in `wrangler.jsonc` and deploy
(or merge — CI applies it). www does not auto-redirect to the apex; add a Cloudflare
redirect rule (or a bulk redirect) if you want `www → apex`.

Serving identical content on `.ai` and `.io` is intentional here. For SEO you may later
add a `<link rel="canonical">` pointing at the primary domain, or redirect `.io → .ai`.
