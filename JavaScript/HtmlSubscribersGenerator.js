var HtmlSubscribersGenerator = /** @class */ (function () {
    function HtmlSubscribersGenerator() {
    }
    HtmlSubscribersGenerator.generateHtml = function (data) {
        return '<h3>Connected Nodes</h3>'
            + this.generateNodesHtml(data.nodes)
            + '<h3>Readers</h3>'
            + this.generateTotalSend(data.readers)
            + this.generateReadersHtml(data.readers)
            + '<h3>Tables</h3>'
            + this.generateTablesHtml(data.tables);
    };
    HtmlSubscribersGenerator.generateTotalSend = function (data) {
        var total = [];
        for (var _i = 0, data_1 = data; _i < data_1.length; _i++) {
            var reader = data_1[_i];
            var i = 0;
            for (var _a = 0, _b = reader.sentPerSecond; _a < _b.length; _a++) {
                var b = _b[_a];
                if (i >= total.length) {
                    total.push(0);
                }
                total[i] += b;
                i += 1;
            }
        }
        return '<div>' + HtmlGraph.renderGraph(total, function (v) { return Utils.format_bytes(v); }, function (v) { return v; }, function (_) { return false; }) + '</div>';
    };
    HtmlSubscribersGenerator.generateReadersHtml = function (data) {
        var html = "<table class=\"table table-striped\"><tr><th>Id</th><th>Client</th><th>Ip</th><th>tables</th><th></th></tr>";
        for (var _i = 0, data_2 = data; _i < data_2.length; _i++) {
            var itm = data_2[_i];
            html += '<tr><td>' + itm.id + '</td><td>' + this.renderName(itm.name) + '</td><td>' + itm.ip + '<div>' + HtmlGraph.renderGraph(itm.sentPerSecond, function (v) { return Utils.format_bytes(v); }, function (v) { return v; }, function (_) { return false; }) + '</div></td><td>' + this.renderTables(itm.tables) + '</td>' +
                '<td style="font-size: 10px">' +
                '<div><b>C:</b>' + itm.connectedTime + '</div>' +
                '<div><b>L:</b>' + itm.lastIncomingTime + '</div>' +
                '<div><b>S:</b>' + itm.pendingToSend + '</div>' +
                '</td></tr>';
        }
        html += '</table>';
        return html;
    };
    HtmlSubscribersGenerator.generateTablesHtml = function (tables) {
        var html = "<table class=\"table table-striped\"><tr><th>Table</th><th>Persist</th><th>DataSize</th><th>Partitions</th><th>Records</th><th>Indexed Records</th><th>Last update</th></tr>";
        var total_size = 0;
        var total_partitions = 0;
        var total_records = 0;
        var total_indexed_records = 0;
        for (var _i = 0, _a = tables.sort(function (a, b) { return a.name > b.name ? 1 : -1; }); _i < _a.length; _i++) {
            var table = _a[_i];
            var style = ' style="color:green" ';
            if (table.lastPersistTime < table.lastUpdateTime) {
                style = ' style="color:red" ';
            }
            var lastUpdateTime = new Date(table.lastUpdateTime / 1000);
            var lastPersistTime = new Date(table.lastPersistTime / 1000);
            var nextPersistTime = "---";
            if (table.nextPersistTime) {
                var as_time = new Date(table.nextPersistTime / 1000);
                nextPersistTime = as_time.toISOString();
            }
            var lineColor = "";
            if (!table.hasCommonThread) {
                lineColor = ' style="background-color: #8bc34a4f" ';
            }
            html += '<tr ' + lineColor + '><td>' + table.name + '</td><td>' + table.persistAmount + '</td><td>' + table.dataSize + '</td><td>' + table.partitionsCount + '</td><td>' + table.recordsAmount + '</td><td>' + table.expirationIndex + '</td>' +
                '<td' + style + '><div>UpdateTime: ' + lastUpdateTime.toISOString() + '</div><div>PersistTime: ' + lastPersistTime.toISOString() + '</div>' +
                '<div>NextPersist: ' + nextPersistTime + '</div>' + HtmlGraph.renderGraph(table.lastPersistDuration, function (v) { return Utils.format_duration(v); }, function (v) { return v; }, function (v) { return false; }) + '</td></tr>';
            total_size += table.dataSize;
            total_partitions += table.partitionsCount;
            total_records += table.recordsAmount;
            total_indexed_records += table.expirationIndex;
        }
        html += '<tr style="font-weight: bold; background-color:black; color:white;"><td>Total</td><td></td><td>DataSize: ' + total_size + '</td><td>Partitions: ' + total_partitions + '</td><td>Records: ' + total_records + '</td><td>Indexed records: ' + total_indexed_records + '</td>'
            + '<td></td></tr>';
        html += '</table>';
        return html;
    };
    HtmlSubscribersGenerator.generateNodesHtml = function (data) {
        var html = "<table class=\"table table-striped\"><tr><th>Location</th><th>Connected</th><th>LastAccess</th><th>Compress</th><th>Latency</th></tr>";
        for (var _i = 0, data_3 = data; _i < data_3.length; _i++) {
            var itm = data_3[_i];
            html += '<tr><td>' + itm.location + '</td><td>' + itm.connected + '</td><td>' + itm.lastAccessed + '</td><td>' + itm.compress + '</td><td>' + itm.latency + '</td></tr>';
        }
        html += '</table>';
        return html;
    };
    HtmlSubscribersGenerator.renderName = function (name) {
        var lines = name.split(';');
        var result = "";
        for (var _i = 0, lines_1 = lines; _i < lines_1.length; _i++) {
            var line = lines_1[_i];
            result += "<div>" + line + "</div>";
        }
        return result;
    };
    HtmlSubscribersGenerator.renderTables = function (data) {
        var result = "";
        for (var _i = 0, data_4 = data; _i < data_4.length; _i++) {
            var itm = data_4[_i];
            result += '<span class="badge badge-info" style="margin-left: 5px">' + itm + '</span>';
        }
        return result;
    };
    return HtmlSubscribersGenerator;
}());
//# sourceMappingURL=HtmlSubscribersGenerator.js.map