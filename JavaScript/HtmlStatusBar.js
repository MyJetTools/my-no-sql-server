var HtmlStatusBar = /** @class */ (function () {
    function HtmlStatusBar() {
    }
    HtmlStatusBar.layout = function () {
        return '<div id="status-bar">' +
            '<table><tr>' +
            '<td style="padding-left: 5px">Connected: <b id="connected" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Location: <b id="location" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Tables: <b id="tables-amount" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Compression: <b id="compression" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Tcp: <b id="tcp-connections" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Http: <b id="http-connections" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Persist queue: <b id="persist-queue" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Sync queue: <b id="sync-queue-size" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '<td>Connected to master node: <b id="master-node" style="text-shadow: 0 0 1px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +
            '</tr></table></div>';
    };
    HtmlStatusBar.updateStatusbar = function (data) {
        if (this.tablesAmount != data.tablesAmount) {
            this.tablesAmount = data.tablesAmount;
            document.getElementById('tables-amount').innerHTML = this.tablesAmount.toString();
        }
        if (!this.connected) {
            this.connected = true;
            document.getElementById('connected').innerHTML = '<span style="color: green">yes</span>';
        }
        if (this.location != data.location.id) {
            document.getElementById('location').innerHTML = data.location.id;
            this.location = data.location.id;
        }
        if (this.compression != data.location.compress) {
            this.compression = data.location.compress;
            document.getElementById('compression').innerHTML = this.compression
                ? '<span style="color: green">enabled</span>'
                : '<span style="color: gray">disabled</span>';
        }
        if (this.masterNode != data.masterNode) {
            this.masterNode = data.masterNode;
            document.getElementById('master-node').innerHTML = this.masterNode
                ? '<span style="color: green">' + this.masterNode + '</span>'
                : '<span style="color: gray">---</span>';
        }
        if (this.tcpConnections != data.tcpConnections) {
            this.tcpConnections = data.tcpConnections;
            document.getElementById('tcp-connections').innerHTML = this.tcpConnections.toString();
        }
        if (this.httpConnections != data.httpConnections) {
            this.httpConnections = data.httpConnections;
            document.getElementById('http-connections').innerHTML = this.httpConnections.toString();
        }
        if (this.persistAmount != data.persistAmount) {
            this.persistAmount = data.persistAmount;
            document.getElementById('persist-queue').innerHTML = this.persistAmount.toString();
        }
    };
    HtmlStatusBar.updateQueueSize = function (queueSize) {
        if (!this.syncQueueSize) {
            this.syncQueueSize = new HtmlStaticElement(document.getElementById('sync-queue-size'));
        }
        this.syncQueueSize.update(queueSize, function (value) { return value.toFixed(0); });
    };
    HtmlStatusBar.updateOffline = function () {
        if (this.connected) {
            this.connected = false;
            document.getElementById('connected').innerHTML = '<span style="color: red">offline</span>';
        }
    };
    return HtmlStatusBar;
}());
//# sourceMappingURL=HtmlStatusBar.js.map