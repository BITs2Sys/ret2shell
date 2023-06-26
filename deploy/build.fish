cd web && pnpm install
pnpm build
cd ../server && podman build -t ret2shell .
