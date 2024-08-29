import '@nomicfoundation/hardhat-toolbox';
import '@openzeppelin/hardhat-upgrades';
import 'hardhat-contract-sizer';

import './tasks/deploy';
import './tasks/upgrade';

import * as defaultConfig from './hardhat.common';

const config = {
  ...defaultConfig.default,
  networks: {
    ethereum: {
      url: "https://rpc.ankr.com/eth"
    },
    holesky: {
      url: 'https://endpoints.omniatech.io/v1/eth/holesky/public',
    },
    holesky2: {
      url: 'https://1rpc.io/holesky',
    }
  },
  etherscan: {
    apiKey: {
      holesky: ""
    }
  },
};

export default config;
