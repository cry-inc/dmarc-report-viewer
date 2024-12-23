import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class Dashboard extends LitElement {
    static styles = [globalStyle, css`
        .container {
            display: grid;
            column-gap: 10px;
            row-gap: 10px;
        }

        .module {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
            text-align: center;
        }

        .module canvas {
            max-width: 300px;
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

        const colorMap = {
            "none": "rgb(108, 117, 125)",
            "fail": "rgb(220, 53, 69)",
            "pass": "rgb(25, 135, 84)",
            "softfail": "rgb(255, 193, 7)",
            "policy": "rgb(13, 110, 253)",
            "neutral": "rgb(13, 202, 240)",
            "temperror": "rgb(253, 126, 20)",
            "permerror": "rgb(132, 32, 41)",
        };

        this.mails = summary.mails;
        this.xmlFiles = summary.xml_files;
        this.reports = summary.reports;
        this.lastUpdate = summary.last_update;

        this.createPieChart("orgs_chart", summary.orgs, null, function (label) {
            window.location.hash = "#/reports?org=" + encodeURIComponent(label);
        });
        this.createPieChart("domains_chart", summary.domains, null, function (label) {
            window.location.hash = "#/reports?domain=" + encodeURIComponent(label);
        });
        this.createPieChart("spf_policy_chart", summary.spf_policy_results, colorMap);
        this.createPieChart("dkim_policy_chart", summary.dkim_policy_results, colorMap);
        this.createPieChart("spf_auth_chart", summary.spf_auth_results, colorMap);
        this.createPieChart("dkim_auth_chart", summary.dkim_auth_results, colorMap);
    }

    async createPieChart(canvasId, dataMap, colorMap, onLabelClick) {
        const element = this.renderRoot.querySelector("." + canvasId);

        const labels = Object.keys(dataMap);
        const data = labels.map(k => dataMap[k]);

        let colors = undefined;
        if (colorMap !== undefined && colorMap !== null) {
            colors = labels.map(l => colorMap[l]);
        } else {
            colors = [
                "rgb(13, 202, 240)",
                "rgb(253, 126, 20)",
                "rgb(25, 135, 84)",
                "rgb(220, 53, 69)",
                "rgb(13, 110, 253)",
                "rgb(255, 193, 7)",
                "rgb(108, 117, 125)",
                "rgb(132, 32, 41)"
            ];
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

            <div class="container">
                <div class="module" style="grid-column: 1; grid-row: 1;">
                    <h2>Organizations</h2>
                    <canvas class="orgs_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 2; grid-row: 1;">
                    <h2>Domains</h2>
                    <canvas class="domains_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 3; grid-row: 1;">
                    <h2>SPF Policy Results</h2>
                    <canvas class="spf_policy_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 1; grid-row: 2;">
                    <h2>DKIM Policy Results</h2>
                    <canvas class="dkim_policy_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 2; grid-row: 2;">
                    <h2>SPF Auth Results</h2>
                    <canvas class="spf_auth_chart"></canvas>
                </div>

                <div class="module" style="grid-column: 3; grid-row: 2;">
                    <h2>DKIM Auth Results</h2>
                    <canvas class="dkim_auth_chart"></canvas>
                </div>
            </div>
        `;
    }
}

customElements.define("dmarc-dashboard", Dashboard);
