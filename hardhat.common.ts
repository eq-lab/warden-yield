import '@nomicfoundation/hardhat-toolbox';
import '@openzeppelin/hardhat-upgrades';
import 'hardhat-contract-sizer';

export const config = {
  solidity: {
    version: '0.8.26',
    settings: {
      optimizer: {
        enabled: true,
        runs: 100000,
      },
    },
  },
  mocha: {
    timeout: 2_000_000,
  },
  networks: {
    ethereum: {
      url: "https://rpc.ankr.com/eth"
    },
    holesky: {
      url: 'https://endpoints.omniatech.io/v1/eth/holesky/public',
    }
  },
  etherscan: {
    apiKey: {
      holesky: ""
    }
  },
  contractSizer: {
    alphaSort: true,
    disambiguatePaths: false,
    runOnCompile: true,
    strict: false,
    only: ['Aave', 'Lido', 'Eigen', 'Yield'],
    except: ['Mock', 'Test'],
  }
};

export default config;
