import { loadDeployConfig } from '../deploy/config';
import hre from 'hardhat';
import { deployWardenYield } from '../deploy/src';

async function main() {
  const privateKey = process.env.CREATOR_PRIVATE_KEY;
  const deployDir = process.env.DEPLOY_DIR;

  const dryRun =
    hre.config.networks.hardhat.forking !== undefined ? hre.config.networks.hardhat.forking.enabled : false;

  if (dryRun) {
    console.log(`Dry run command on fork`);
    const blockNumber = await hre.ethers.provider.getBlockNumber();
    console.log(`Fork block number: ${blockNumber}`);
  }

  const signer = new hre.ethers.Wallet(privateKey, hre.ethers.provider);
  const config = await loadDeployConfig(deployDir, signer.provider!, dryRun);

  await deployWardenYield(signer, config, deployDir, dryRun, hre);

  console.log(`Done!`);
}

main();
