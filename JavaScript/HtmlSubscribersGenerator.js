var HtmlSubscribersGenerator = /** @class */ (function () {
    function HtmlSubscribersGenerator() {
    }
    HtmlSubscribersGenerator.generateHtml = function (data) {
        var nodes = [];
        var readers = [];
        for (var _i = 0, _a = data.readers; _i < _a.length; _i++) {
            var reader = _a[_i];
            if (reader.isNode) {
                nodes.push(reader);
            }
            else {
                readers.push(reader);
            }
        }
        return '<h3>Connected Nodes</h3>'
            + this.generateReadersHtml(nodes)
            + '<h3>Readers</h3>'
            + this.generateTotalSend(data.readers)
            + this.generateReadersHtml(readers)
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
        for (var _i = 0, _a = data.sort(function (a, b) { return a.connectedTime > b.connectedTime ? 1 : -1; }); _i < _a.length; _i++) {
            var reader = _a[_i];
            html += this.generateReader(reader);
        }
        html += '</table>';
        return html;
    };
    HtmlSubscribersGenerator.generateReader = function (reader) {
        return '<tr><td>' + reader.id + '</td><td>' + this.renderName(reader.name) + '</td><td>' + reader.ip + '<div>' + HtmlGraph.renderGraph(reader.sentPerSecond, function (v) { return Utils.format_bytes(v); }, function (v) { return v; }, function (_) { return false; }) + '</div></td><td>' + this.renderTables(reader.tables) + '</td>' +
            '<td style="font-size: 10px">' +
            '<div><b>C:</b>' + reader.connectedTime + '</div>' +
            '<div><b>L:</b>' + reader.lastIncomingTime + '</div>' +
            '<div><b>S:</b>' + reader.pendingToSend + '</div>' +
            '</td></tr>';
    };
    HtmlSubscribersGenerator.generateTablesHtml = function (tables) {
        var html = "<table class=\"table table-striped\"><tr><th>Table</th><th>Persist</th><th>DataSize</th><th>Avg entity size</th><th>Partitions</th><th>Records</th><th>Indexed Records</th><th>Last update</th></tr>";
        var total_size = 0;
        var total_partitions = 0;
        var total_records = 0;
        var total_indexed_records = 0;
        for (var _i = 0, _a = tables.sort(function (a, b) { return a.name > b.name ? 1 : -1; }); _i < _a.length; _i++) {
            var table = _a[_i];
            var style = ' style="color:green" ';
            if (!table.lastPersistTime) {
                style = ' style="color:gray" ';
            }
            else if (table.lastPersistTime < table.lastUpdateTime) {
                style = ' style="color:red" ';
            }
            var lastUpdateTime = new Date(table.lastUpdateTime / 1000);
            var lastPersistTime = "----";
            if (table.lastPersistTime) {
                lastPersistTime = new Date(table.lastPersistTime / 1000).toISOString();
            }
            var nextPersistTime = "---";
            if (table.nextPersistTime) {
                var as_time = new Date(table.nextPersistTime / 1000);
                nextPersistTime = as_time.toISOString();
            }
            var persist_badge = table.persist ? '<span class="badge badge-success">Persist</span>' : '<span class="badge badge-primary">Not persisted</span>';
            var max_partitions_amount = table.maxPartitionsAmount ? '<span class="badge badge-success">Max partitions: ' + table.maxPartitionsAmount + '</span>' : '<span class="badge badge-primary">Max partitions: Unlimited</span>';
            var max_rows_per_partition = table.maxRowsPerPartition ? '<span class="badge badge-success">Max rows per partition: ' + table.maxRowsPerPartition + '</span>' : '<span class="badge badge-primary">Max rows per partition: Unlimited</span>';
            html += '<tr><td>' + table.name + '<div>' + persist_badge + '</div><div>' + max_partitions_amount + '</div><div>' + max_rows_per_partition + '</div></td><td>' + table.persistAmount + '</td><td>' + Utils.formatNumber(table.dataSize) + '</td><td>' + Utils.formatNumber(table.avgEntitySize) + '</td><td>' + Utils.formatNumber(table.partitionsCount) + '</td><td>' + Utils.formatNumber(table.recordsAmount) + '</td><td>' + Utils.formatNumber(table.expirationIndex) + '</td>' +
                '<td' + style + '><div>UpdateTime: ' + lastUpdateTime.toISOString() + '</div><div>PersistTime: ' + lastPersistTime + '</div>' +
                '<div>NextPersist: ' + nextPersistTime + '</div>' + HtmlGraph.renderGraph(table.lastPersistDuration, function (v) { return Utils.format_duration(v); }, function (v) { return v; }, function (v) { return false; }) + '</td></tr>';
            total_size += table.dataSize;
            total_partitions += table.partitionsCount;
            total_records += table.recordsAmount;
            total_indexed_records += table.expirationIndex;
        }
        $('#total-data-size').html(Utils.formatNumber(total_size));
        html += '<tr style="font-weight: bold; background-color:black; color:white;"><td>Total</td><td></td><td>DataSize: ' + Utils.formatNumber(total_size) + '</td><td></td><td>Partitions: ' + Utils.formatNumber(total_partitions) + '</td><td>Records: ' + Utils.formatNumber(total_records) + '</td><td>Indexed records: ' + Utils.formatNumber(total_indexed_records) + '</td>'
            + '<td></td></tr>';
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
        for (var _i = 0, data_2 = data; _i < data_2.length; _i++) {
            var itm = data_2[_i];
            result += '<span class="badge badge-info" style="margin-left: 5px">' + itm + '</span>';
        }
        return result;
    };
    return HtmlSubscribersGenerator;
}());
//# sourceMappingURL=HtmlSubscribersGenerator.js.map