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
        params: { type: Object },
        mails: { type: Number },
        xmlFiles: { type: Number },
        dmarcReports: { type: Number },
        lastUpdate: { type: Number },
        domains: { type: Array },
    };

    constructor() {
        super();

        this.params = {};
        this.mails = 0;
        this.xmlFiles = 0;
        this.dmarcReports = 0;
        this.lastUpdate = 0;
        this.domains = [];

        this.getDomains();
    }

    async getDomains() {
        const response = await fetch("summary");
        const summary = await response.json();
        this.domains = Object.keys(summary.dmarc_domains);
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateCharts();
        }
    }

    onTimeSpanChange(event) {
        const value = event.target.value;
        if (value && value !== "0") {
            this.params.ts = value;
        } else {
            delete this.params.ts;
        }
        this.updateByParams();
    }

    onDomainChange(event) {
        const value = event.target.value;
        if (value && value !== "all") {
            this.params.domain = value;
        } else {
            delete this.params.domain;
        }
        this.updateByParams();
    }

    updateByParams() {
        let params = Object.
            keys(this.params).
            map(k => k + "=" + this.params[k]).
            join("&");
        if (params.length > 0) {
            document.location.href = "#/dashboard?" + params;
        } else {
            document.location.href = "#/dashboard";
        }
    }

    async updateCharts() {
        const queryParams = [];
        if (this.params.ts && this.params.ts !== "0") {
            queryParams.push("time_span=" + this.params.ts);
        }
        if (this.params.domain && this.params.domain !== "all") {
            queryParams.push("domain=" + this.params.domain);
        }
        let url = "summary";
        if (queryParams.length > 0) {
            url += "?" + queryParams.join("&");
        }
        const response = await fetch(url);
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
        this.dmarcReports = summary.dmarc_reports;
        this.lastUpdate = summary.last_update;

        if (this.orgs_chart) this.orgs_chart.destroy();
        this.orgs_chart = await this.createPieChart("orgs_chart", this.sortedMap(summary.dmarc_orgs), orgColorMap, function (label) {
            window.location.hash = "#/dmarc-reports?org=" + encodeURIComponent(label);
        });

        if (this.domains_chart) this.domains_chart.destroy();
        this.domains_chart = await this.createPieChart("domains_chart", this.sortedMap(summary.dmarc_domains), null, function (label) {
            window.location.hash = "#/dmarc-reports?domain=" + encodeURIComponent(label);
        });

        if (this.spf_policy_chart) this.spf_policy_chart.destroy();
        this.spf_policy_chart = await this.createPieChart("spf_policy_chart", this.sortedMap(summary.spf_policy_results), resultColorMap);

        if (this.dkim_policy_chart) this.dkim_policy_chart.destroy();
        this.dkim_policy_chart = await this.createPieChart("dkim_policy_chart", this.sortedMap(summary.dkim_policy_results), resultColorMap);

        if (this.spf_auth_chart) this.spf_auth_chart.destroy();
        this.spf_auth_chart = await this.createPieChart("spf_auth_chart", this.sortedMap(summary.spf_auth_results), resultColorMap);

        if (this.dkim_auth_chart) this.dkim_auth_chart.destroy();
        this.dkim_auth_chart = await this.createPieChart("dkim_auth_chart", this.sortedMap(summary.dkim_auth_results), resultColorMap);
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

        return new Chart(element, {
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
                <span>DMARC Reports: <b>${this.dmarcReports}</b></span>
                <span>Last Update: <b>${new Date(this.lastUpdate * 1000).toLocaleString()}</b></span>
            </div>

            <div class="module stats">
                <span>
                    Time Span for Summary Charts:
                    <select @change="${this.onTimeSpanChange}">
                        <option value="0">Everything</option>
                        <option ?selected=${this.params.ts === "72"} value="72">Last Three Days</option>
                        <option ?selected=${this.params.ts === "168"} value="168">Last Week</option>
                        <option ?selected=${this.params.ts === "744"} value="744">Last Month</option>
                        <option ?selected=${this.params.ts === "4464"} value="4464">Last Six Months</option>
                        <option ?selected=${this.params.ts === "8760"} value="8760">Last Year</option>
                    </select>
                </span>

                <span>
                    Domain:
                    <select @change="${this.onDomainChange}">
                        <option value="all">All</option>
                        ${this.domains.map((domain) =>
                        html`<option
                                ?selected=${this.params.domain === encodeURIComponent(domain)}
                                value="${encodeURIComponent(domain)}">${domain}</option>`
                        )}
                    </select>
                </span>
            </div>

            <div class="grid">
                <div class="module">
                    <h2>DMARC Organizations</h2>
                    <canvas class="orgs_chart"></canvas>
                </div>

                <div class="module">
                    <h2>DMARC Domains</h2>
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

customElements.define("drv-dashboard", Dashboard);
