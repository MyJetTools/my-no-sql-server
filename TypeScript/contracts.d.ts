interface IReaderStatus {
    id: number;
    ip: string;
    name: string;
    tables: string[];
    connectedTime: string;
    lastIncomingTime: string;
    pendingToSend: number;
    sentPerSecond: number[];
    isNode: boolean;
}

interface ILocationStatus {
    id: string,
    compress: boolean
}

interface INonInitializedModel {
    tablesTotal: number,
    tablesLoaded: number,
    filesTotal: number,
    filesLoaded: number,
    initializingSeconds: number,
}



interface IStatus {
    notInitialized: INonInitializedModel,
    initialized: IInitializedStatus
    statusBar: IStatusBarModel;

}

interface IInitializedStatus {
    readers: IReaderStatus[],
    writers: IWriterStatus[],
    tables: ITableModel[]
}

interface IStatusBarModel {
    persistAmount: number;
    tcpConnections: number;
    tablesAmount: number;
    httpConnections: number;
    location: ILocationStatus,
    masterNode: string,
    syncQueueSize: number,
    usedHttpConnections: number,
}

interface ITableModel {
    name: number;
    persist: boolean;
    maxPartitionsAmount?: number;
    maxRowsPerPartition?: number;
    partitionsCount: number;
    dataSize: number;
    recordsAmount: number;
    expirationIndex: number;
    lastUpdateTime: number;
    lastPersistTime: number;
    nextPersistTime: number;
    lastPersistDuration: number[];
    persistAmount: number;
    avgEntitySize: number
}

interface IWriterStatus {
    name: string;
    version: string;
    last_update: string;
    tables: string[];
}