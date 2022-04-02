var HtmlSubscribersGenerator = /** @class */ (function () {
    function HtmlSubscribersGenerator() {
    }
    HtmlSubscribersGenerator.generateHtml = function (data) {
        return '<h3>Connected Nodes</h3>'
            + this.generateNodesHtml(data.nodes)
            + '<h3>Readers</h3>'
            + this.generateReadersHtml(data.readers)
            + '<h3>Tables</h3>'
            + this.generateTablesHtml(data.tables);
    };
    HtmlSubscribersGenerator.generateReadersHtml = function (data) {
        var html = "<table class=\"table table-striped\"><tr><th>Id</th><th>Client</th><th>Ip</th><th>tables</th><th></th></tr>";
        for (var _i = 0, data_1 = data; _i < data_1.length; _i++) {
            var itm = data_1[_i];
            html += '<tr><td>' + itm.id + '</td><td>' + this.renderName(itm.name) + '</td><td>' + itm.ip + '</td><td>' + this.renderTables(itm.tables) + '</td>' +
                '<td style="font-size: 10px">' +
                '<div><b>C:</b>' + itm.connectedTime + '</div>' +
                '<div><b>L:</b>' + itm.lastIncomingTime + '</div>' +
                '</td></tr>';
        }
        html += '</table>';
        return html;
    };
    HtmlSubscribersGenerator.generateTablesHtml = function (tables) {
        var html = "<table class=\"table table-striped\"><tr><th>Table</th><th>DataSize</th><th>Partitions</th><th>Records</th><th>Indexed Records</th></tr>";
        var total_size = 0;
        var total_partitions = 0;
        var total_records = 0;
        var total_indexed_records = 0;
        for (var _i = 0, tables_1 = tables; _i < tables_1.length; _i++) {
            var table = tables_1[_i];
            html += '<tr><td>' + table.name + '</td><td>' + table.dataSize + '</td><td>' + table.partitionsCount + '</td><td>' + table.recordsAmount + '</td><td>' + table.expirationIndex + '</td>'
                + '</tr>';
        }
        html += '<tr style="font-weight: bold; background-color:black; color:white;"><td>Total</td><td>DataSize: ' + total_size + '</td><td>Partitions: ' + total_partitions + '</td><td>Records: ' + total_records + '</td><td>Indexed records: ' + total_indexed_records + '</td>'
            + '</tr>';
        html += '</table>';
        return html;
    };
    HtmlSubscribersGenerator.generateNodesHtml = function (data) {
        var html = "<table class=\"table table-striped\"><tr><th>Location</th><th>Connected</th><th>LastAccess</th><th>Compress</th><th>Latency</th></tr>";
        for (var _i = 0, data_2 = data; _i < data_2.length; _i++) {
            var itm = data_2[_i];
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
        for (var _i = 0, data_3 = data; _i < data_3.length; _i++) {
            var itm = data_3[_i];
            result += '<span class="badge badge-info" style="margin-left: 5px">' + itm + '</span>';
        }
        return result;
    };
    return HtmlSubscribersGenerator;
}());
//# sourceMappingURL=HtmlSubscribersGenerator.js.map