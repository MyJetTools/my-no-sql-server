interface IReaderStatus {
    id: number;
    ip: string;
    name: string;
    tables: string[];
    connectedTime: string;
    lastIncomingTime: string;
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
    tablesRemains: number,
    initializingSeconds: number,
    progress: ITableLoadProgress[]
}


interface ITableLoadProgress {
    tableName: String,
    loaded: number,
    partitions: number,
    secondsGone: number
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
    persistAmount: number;
    hasCommonThread: boolean;
}