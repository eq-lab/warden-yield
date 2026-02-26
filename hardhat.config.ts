import '@nomicfoundation/hardhat-toolbox';
import '@openzeppelin/hardhat-upgrades';
import 'hardhat-contract-sizer';

import './tasks/deploy';
import './tasks/deploy-AaveYield-v2-impl';
import './tasks/upgrade-AaveYield-to-v2';
import './tasks/test-upgrade-AaveYield-to-v2';

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
