var HtmlMain = /** @class */ (function () {
    function HtmlMain() {
    }
    HtmlMain.layout = function () {
        return '<div id="main"></div>' +
            HtmlStatusBar.layout()
            + HtmlDialog.layout();
    };
    HtmlMain.generateInit = function (model) {
        var result = '<h1>Remains tables to load: ' + model.tablesRemains + '</h1><h2>Total loading time is: ' + this.formatSeconds(model.initializingSeconds) + '</h2>' +
            '<table class="table table-striped table-bordered"><tr><th>TableName</th><th>Partitions loaded</th><th>Partitions total</th><th>Time gone</th><th>Time estimation</th></tr>';
        for (var _i = 0, _a = model.progress.sort(function (a, b) { return a.tableName > b.tableName ? 1 : -1; }); _i < _a.length; _i++) {
            var itm = _a[_i];
            result += '<tr><td style="width:50%">' + itm.tableName + '</td><td>' + itm.loaded + '</td><td>' + itm.toLoad + "</td><td>" + this.formatSeconds(itm.secondsGone) + "</td><td>" + this.getInitRemains(itm) + "</td></tr>";
        }
        return result + "</table>";
    };
    HtmlMain.getInitRemains = function (progress) {
        if (progress.toLoad == 0 || progress.loaded == 0) {
            return "Unknown";
        }
        var pieceDuration = progress.secondsGone / progress.loaded;
        var remains = (progress.toLoad - progress.loaded) * pieceDuration;
        remains = this.trunc(remains);
        return this.formatSeconds(remains);
    };
    HtmlMain.formatSecMin = function (value) {
        if (value < 10) {
            return "0" + value.toFixed(0);
        }
        return value.toFixed(0);
    };
    HtmlMain.trunc = function (value) {
        var result = value.toFixed(2);
        var pos = result.indexOf('.');
        if (pos < 0) {
            pos = result.indexOf(',');
        }
        return parseInt(result.substring(0, pos));
    };
    HtmlMain.formatSeconds = function (seconds) {
        var hours = 0;
        if (seconds >= 3600) {
            hours = this.trunc(seconds / 3600);
            seconds -= hours * 3600;
        }
        var mins = 0;
        if (seconds >= 60) {
            mins = this.trunc(seconds / 60);
            seconds -= mins * 60;
        }
        return this.formatSecMin(hours) + ":" + this.formatSecMin(mins) + ":" + this.formatSecMin(seconds);
    };
    return HtmlMain;
}());
//# sourceMappingURL=HtmlMain.js.map