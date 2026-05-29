const fs = require('node:fs/promises');

module.exports = async (_forgeConfig, { outputPaths }) => {
  const packageDir = outputPaths.find((outputPath) => outputPath.includes('-linux-'));

  if (!packageDir) {
    throw new Error(`Packaged Goose app directory not found in output paths: ${outputPaths.join(', ')}`);
  }

  await fs.rm('/app/lib/goose', { recursive: true, force: true });
  await fs.cp(packageDir, '/app/lib/goose', { recursive: true });
};
