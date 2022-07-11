interface IReaderStatus {
    id: number;
    ip: string;
    name: string;
    tables: string[];
    connectedTime: string;
    lastIncomingTime: string;
    pendingToSend: number;
    sentPerSecond: number[];
}

interface INodeStatus {
    location: string,
    lastAccessed: string,
    connected: string,
    compress: boolean
    latency: string
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
    nodes: INodeStatus[]
    tables: ITableModel[]
}

interface IStatusBarModel {
    persistAmount: number;
    tcpConnections: number;
    tablesAmount: number;
    httpConnections: number;
    location: ILocationStatus,
    masterNode: string,
    syncQueueSize: number
}

interface ITableModel {
    name: number;
    partitionsCount: number;
    dataSize: number;
    recordsAmount: number;
    expirationIndex: number;
    lastUpdateTime: number;
    lastPersistTime: number;
    nextPersistTime: number;
    lastPersistDuration: number[];
    persistAmount: number;
    hasCommonThread: boolean;
}