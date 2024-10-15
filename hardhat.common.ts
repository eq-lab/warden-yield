import '@nomicfoundation/hardhat-toolbox';
import '@openzeppelin/hardhat-upgrades';
import 'hardhat-contract-sizer';

export const config = {
  solidity: {
    compilers: [
      {
        version: '0.8.26',
        settings: {
          viaIR: true,
          optimizer: {
            enabled: true,
            runs: 10000,
          },
        },
      },
      {
        version: '0.4.18',
        settings: {
          optimizer: {
            enabled: false,
            runs: 200,
          },
        },
        // config for TestWETH9 contract
      },
    ],
  },
  mocha: {
    timeout: 2_000_000,
  },
  contractSizer: {
    alphaSort: true,
    disambiguatePaths: false,
    runOnCompile: true,
    strict: false,
    only: ['Aave', 'Lido', 'Eigen', 'Yield'],
    except: ['Mock', 'Test'],
  },
};

export default config;
