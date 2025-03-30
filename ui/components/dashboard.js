import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class Dashboard extends LitElement {
    static styles = [globalStyle, css`
        .grid {
            display: grid;
            gap: 10px;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        }

        .module {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
            text-align: center;
        }

        .module canvas {
            margin: auto;
        }

        .stats {
            margin-bottom: 10px;
        }

        .stats span {
            margin-left: 15px;
            margin-right: 15px;
        }
    `];

    static properties = {
        mails: { type: Number },
        xmlFiles: { type: Number },
        reports: { type: Number },
        lastUpdate: { type: Number },
    };

    constructor() {
        super();

        this.mails = 0;
        this.xmlFiles = 0;
        this.reports = 0;
        this.lastUpdate = 0;
    }

    async firstUpdated() {
        const response = await fetch("summary");
        const summary = await response.json();

        const resultColorMap = {
            "none": "rgb(108, 117, 125)",
            "fail": "rgb(220, 53, 69)",
            "pass": "rgb(25, 135, 84)",
            "softfail": "rgb(255, 193, 7)",
            "policy": "rgb(13, 110, 253)",
            "neutral": "rgb(13, 202, 240)",
            "temperror": "rgb(253, 126, 20)",
            "permerror": "rgb(132, 32, 41)",
        };

        const orgColorMap = {
            "google.com": "#ea4335",
            "Yahoo": "#6001d2",
            "WEB.DE": "#ffd800",
            "Mail.Ru": "#0078ff",
            "GMX": "#1c449b",
            "Outlook.com": "#0078d4",
            "Enterprise Outlook": "#0078d4",
            "Fastmail Pty Ltd": "#0067b9",
            "AMAZON-SES": "#ff9900",
        };

        this.mails = summary.mails;
        this.xmlFiles = summary.xml_files;
        this.reports = summary.reports;
        this.lastUpdate = summary.last_update;

        this.createPieChart("orgs_chart", this.sortedMap(summary.orgs), orgColorMap, function (label) {
            window.location.hash = "#/reports?org=" + encodeURIComponent(label);
        });
        this.createPieChart("domains_chart", this.sortedMap(summary.domains), null, function (label) {
            window.location.hash = "#/reports?domain=" + encodeURIComponent(label);
        });
        this.createPieChart("spf_policy_chart", this.sortedMap(summary.spf_policy_results), resultColorMap);
        this.createPieChart("dkim_policy_chart", this.sortedMap(summary.dkim_policy_results), resultColorMap);
        this.createPieChart("spf_auth_chart", this.sortedMap(summary.spf_auth_results), resultColorMap);
        this.createPieChart("dkim_auth_chart", this.sortedMap(summary.dkim_auth_results), resultColorMap);
    }

    sortedMap(map) {
        const keys = Object.keys(map);
        keys.sort((a, b) => {
            if (map[a] < map[b])
                return 1;
            if (map[a] > map[b])
                return -1;
            else
                return b;
        });
        const newMap = {};
        keys.forEach(k => newMap[k] = map[k]);
        return newMap;
    }

    async createPieChart(canvasId, dataMap, colorMap, onLabelClick) {
        const defaultColors = [
            "rgb(13, 202, 240)",
            "rgb(253, 126, 20)",
            "rgb(25, 135, 84)",
            "rgb(220, 53, 69)",
            "rgb(13, 110, 253)",
            "rgb(255, 193, 7)",
            "rgb(108, 117, 125)",
            "rgb(132, 32, 41)"
        ];

        const element = this.renderRoot.querySelector("." + canvasId);

        const labels = Object.keys(dataMap);
        const data = labels.map(k => dataMap[k]);

        let colors = undefined;
        if (colorMap !== undefined && colorMap !== null) {
            colors = labels.map(l => colorMap[l]);

            // Use default color set to colorize labels without explicit color
            let nextColor = 0;
            for (let i = 0; i < colors.length; i++) {
                if (!colors[i]) {
                    colors[i] = defaultColors[nextColor % defaultColors.length];
                    nextColor++;
                }
            }
        } else {
            colors = defaultColors;
        }

        new Chart(element, {
            type: "pie",
            data: {
                labels,
                datasets: [{
                    data: data,
                    backgroundColor: colors
                }],
            },
            options: {
                onClick: function (event, element, chart) {
                    if (onLabelClick) {
                        const label = labels[element[0].index];
                        onLabelClick(label);
                    }
                },
                plugins: {
                    legend: {
                        maxHeight: 70
                    }
                }
            }
        });
    }

    render() {
        return html`
            <h1>Dashboard</h1>
            <div class="module stats">
                <span>Mails: <b>${this.mails}</b></span>
                <span>XML Files: <b>${this.xmlFiles}</b></span>
                <span>DMARC Reports: <b>${this.reports}</b></span>
                <span>Last Update: <b>${new Date(this.lastUpdate * 1000).toLocaleString()}</b></span>
            </div>

            <div class="grid">
                <div class="module">
                    <h2>Organizations</h2>
                    <canvas class="orgs_chart"></canvas>
                </div>

                <div class="module">
                    <h2>Domains</h2>
                    <canvas class="domains_chart"></canvas>
                </div>

                <div class="module">
                    <h2>SPF Policy Results</h2>
                    <canvas class="spf_policy_chart"></canvas>
                </div>

                <div class="module">
                    <h2>DKIM Policy Results</h2>
                    <canvas class="dkim_policy_chart"></canvas>
                </div>

                <div class="module">
                    <h2>SPF Auth Results</h2>
                    <canvas class="spf_auth_chart"></canvas>
                </div>

                <div class="module">
                    <h2>DKIM Auth Results</h2>
                    <canvas class="dkim_auth_chart"></canvas>
                </div>
            </div>
        `;
    }
}

customElements.define("dmarc-dashboard", Dashboard);
