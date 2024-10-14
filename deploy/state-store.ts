import fs from 'fs';
import path from 'path';
import { Logger } from './logger';

export interface StateStore {
  getById: (id: string) => DeployState | undefined;
  setById: (id: string, deployState: DeployState) => void;
  stringify: () => string;
}

export interface DeployState {
  address: string;
  txHash?: string;
}

export interface BaseState {
  contracts: {
    [id: string]: DeployState;
  };
}

export function createDefaultBaseState(): BaseState {
  return { contracts: {} };
}

export class StateFile<TState extends BaseState> {
  private readonly stateName: string;
  private readonly createDefaultState: () => TState;
  private readonly fileName: string;
  private readonly writeToFile: boolean;
  private readonly logger: Logger;

  constructor(name: string, createDefaultState: () => TState, fileName: string, writeToFile: boolean, logger: Logger) {
    this.stateName = name;
    this.createDefaultState = createDefaultState;
    this.fileName = fileName;
    this.writeToFile = writeToFile;
    this.logger = logger;
  }

  public getStateFromFile(): TState {
    if (fs.existsSync(this.fileName)) {
      return JSON.parse(fs.readFileSync(this.fileName, 'utf-8'));
    } else {
      this.logger.log(`${this.stateName} state file not found, a new one will created`);
    }

    return this.createDefaultState();
  }

  public saveStateFileChanges(state: TState) {
    const s = JSON.stringify(state, null, 2);
    fs.writeFileSync(this.fileName, s, { encoding: 'utf8' });
  }

  public createStateStore(): StateStore {
    const state = this.getStateFromFile();
    return {
      getById: (id: string): DeployState | undefined => {
        return state.contracts[id];
      },
      setById: (id: string, deployState: DeployState) => {
        state.contracts[id] = deployState;
        if (this.writeToFile) {
          this.saveStateFileChanges(state);
        }
      },
      stringify: (): string => {
        return JSON.stringify(state.contracts, null, 2);
      },
    };
  }
}

export function getStateFileName(network: string, statesDirName: string): string {
  const dirName = path.join(__dirname, `data`, `configs`, network, statesDirName);
  if (!fs.existsSync(dirName)) {
    fs.mkdirSync(dirName);
  }

  if (!fs.statSync(dirName).isDirectory()) {
    throw new Error(`Not a directory: ${dirName}`);
  }

  let stateFilename = getLatestStateFileName(dirName);

  if (stateFilename === undefined) {
    const fileName = path.join(statesDirName, generateStateFileName(dirName));
    console.log(`Using new generated state file '${fileName}'`);
    return fileName;
  } else {
    const fileName = path.join(statesDirName, stateFilename);
    console.log(`Using latest state file '${fileName}'`);
    return fileName;
  }
}

export function getLatestStateFileName(dirName: string): string | undefined {
  const fileNames = fs.readdirSync(dirName);
  const files = fileNames
    .map((x) => ({
      name: x,
      extension: path.extname(x),
      mtimeNs: fs.statSync(path.join(dirName, x), { bigint: true }).mtimeNs,
    }))
    .filter((x) => x.extension === '.json')
    .sort((a, b) => Number(b.mtimeNs - a.mtimeNs));

  if (files.length === 0) {
    return undefined;
  }

  return files[0].name;
}

export function generateStateFileName(dirName: string): string {
  const now = new Date();
  const year = now.getUTCFullYear();
  const month = 1 + now.getUTCMonth();
  const day = now.getUTCDate();
  const dateStr = `${year}-${month.toString().padStart(2, '0')}-${day.toString().padStart(2, '0')}`;

  let fileName = `${dateStr}.json`;
  if (fs.existsSync(path.join(dirName, fileName))) {
    const maxCount = 99;
    let n = 1;
    while (fs.existsSync(path.join(dirName, fileName))) {
      if (n === maxCount) {
        throw new Error('Too much state files today');
      }
      fileName = `${dateStr}_${n.toString().padStart(2, '0')}.json`;
      n++;
    }
  }
  return fileName;
}
