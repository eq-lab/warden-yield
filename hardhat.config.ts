import '@nomicfoundation/hardhat-toolbox';
import '@openzeppelin/hardhat-upgrades';
import 'hardhat-contract-sizer';

import './tasks/deploy';

import { config } from './hardhat.common';

export default config;
