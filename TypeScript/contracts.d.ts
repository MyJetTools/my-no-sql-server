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

}

interface IInitializedStatus {
    masterNode: string,
    tablesAmount: number,
    location: ILocationStatus,
    readers: IReaderStatus[],
    nodes: INodeStatus[]
    tcpConnections: number,
}