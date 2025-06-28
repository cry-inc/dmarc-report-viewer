import { LitElement, html, css } from "lit";
import { globalStyle } from "../style.js";

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
        jsonFiles: { type: Number },
        dmarcReports: { type: Number },
        tlsReports: { type: Number },
        lastUpdate: { type: Number },
        dmarcDomains: { type: Array },
        tlsDomains: { type: Array },
        classesToHide: { type: Array },
    };

    constructor() {
        super();

        this.params = {};
        this.mails = 0;
        this.xmlFiles = 0;
        this.jsonFiles = 0;
        this.dmarcReports = 0;
        this.tlsReports = 0;
        this.lastUpdate = 0;
        this.dmarcDomains = [];
        this.tlsDomains = [];
        this.filterDomains = [];

        this.getDomains();
    }

    async getDomains() {
        const response = await fetch("summary");
        const summary = await response.json();
        this.dmarcDomains = Object.keys(summary.dmarc.domains);
        this.dmarcDomains.sort();
        this.tlsDomains = Object.keys(summary.tls.domains);
        this.tlsDomains.sort();
        this.filterDomains = [...new Set([
            ...this.dmarcDomains,
            ...this.tlsDomains
        ])].sort();
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
            // DMARC result colors
            "none": "rgb(108, 117, 125)",
            "fail": "rgb(220, 53, 69)",
            "pass": "rgb(25, 135, 84)",
            "softfail": "rgb(255, 193, 7)",
            "policy": "rgb(13, 110, 253)",
            "neutral": "rgb(13, 202, 240)",
            "temperror": "rgb(253, 126, 20)",
            "permerror": "rgb(132, 32, 41)",
            // SMTP TLS result colors
            "failure": "rgb(220, 53, 69)",
            "successful": "rgb(25, 135, 84)",
        };

        const orgColorMap = {
            // DMARC organization colors
            "google.com": "#ea4335",
            "Yahoo": "#6001d2",
            "WEB.DE": "#ffd800",
            "Mail.Ru": "#0078ff",
            "GMX": "#1c449b",
            "Outlook.com": "#0078d4",
            "Enterprise Outlook": "#0078d4",
            "Fastmail Pty Ltd": "#0067b9",
            "AMAZON-SES": "#ff9900",
            // SMTP TLS organization colors
            "Microsoft Corporation": "#0078d4",
            "Google Inc.": "#ea4335",
        };

        this.mails = summary.mails;
        this.xmlFiles = summary.dmarc.files;
        this.jsonFiles = summary.tls.files;
        this.dmarcReports = summary.dmarc.reports;
        this.tlsReports = summary.tls.reports;
        this.lastUpdate = summary.last_update;
        this.classesToHide = [];

        if (Object.values(summary.dmarc.orgs).every((v) => v === 0)) this.classesToHide.push("dmarc_orgs");
        if (Object.values(summary.dmarc.domains).every((v) => v === 0)) this.classesToHide.push("dmarc_domains");
        if (Object.values(summary.dmarc.spf_policy_results).every((v) => v === 0)) this.classesToHide.push("spf_policy");
        if (Object.values(summary.dmarc.dkim_policy_results).every((v) => v === 0)) this.classesToHide.push("dkim_policy");
        if (Object.values(summary.dmarc.spf_auth_results).every((v) => v === 0)) this.classesToHide.push("spf_auth");
        if (Object.values(summary.dmarc.dkim_auth_results).every((v) => v === 0)) this.classesToHide.push("dkim_auth");
        const allDmarcCharts = ["dmarc_orgs", "dmarc_domains", "spf_policy", "dkim_policy", "spf_auth", "dkim_auth"];
        if (allDmarcCharts.every(c => this.classesToHide.includes(c))) this.classesToHide.push("dmarc_charts");

        if (Object.values(summary.tls.orgs).every((v) => v === 0)) this.classesToHide.push("tls_orgs");
        if (Object.values(summary.tls.domains).every((v) => v === 0)) this.classesToHide.push("tls_domains");
        if (Object.values(summary.tls.policy_types).every((v) => v === 0)) this.classesToHide.push("tls_policy_types");
        if (Object.values(summary.tls.sts_policy_results).every((v) => v === 0)) this.classesToHide.push("sts_policy_results");
        if (Object.values(summary.tls.sts_failure_types).every((v) => v === 0)) this.classesToHide.push("sts_failure_types");
        if (Object.values(summary.tls.tlsa_policy_results).every((v) => v === 0)) this.classesToHide.push("tlsa_policy_results");
        if (Object.values(summary.tls.tlsa_failure_types).every((v) => v === 0)) this.classesToHide.push("tlsa_failure_types");
        const allTlsCharts = ["tls_orgs", "tls_domains", "tls_policy_types", "sts_policy_results", "sts_failure_types", "tlsa_policy_results", "tlsa_failure_types"];
        if (allTlsCharts.every(c => this.classesToHide.includes(c))) this.classesToHide.push("tls_charts");

        if (this.dmarc_orgs_chart) this.dmarc_orgs_chart.destroy();
        this.dmarc_orgs_chart = await this.createPieChart("dmarc_orgs_chart", this.sortedMap(summary.dmarc.orgs), orgColorMap, function (label) {
            window.location.hash = "#/dmarc-reports?org=" + encodeURIComponent(label);
        });

        if (this.dmarc_domains_chart) this.dmarc_domains_chart.destroy();
        this.dmarc_domains_chart = await this.createPieChart("dmarc_domains_chart", this.sortedMap(summary.dmarc.domains), null, function (label) {
            window.location.hash = "#/dmarc-reports?domain=" + encodeURIComponent(label);
        });

        if (this.spf_policy_chart) this.spf_policy_chart.destroy();
        this.spf_policy_chart = await this.createPieChart("spf_policy_chart", this.sortedMap(summary.dmarc.spf_policy_results), resultColorMap);

        if (this.dkim_policy_chart) this.dkim_policy_chart.destroy();
        this.dkim_policy_chart = await this.createPieChart("dkim_policy_chart", this.sortedMap(summary.dmarc.dkim_policy_results), resultColorMap);

        if (this.spf_auth_chart) this.spf_auth_chart.destroy();
        this.spf_auth_chart = await this.createPieChart("spf_auth_chart", this.sortedMap(summary.dmarc.spf_auth_results), resultColorMap);

        if (this.dkim_auth_chart) this.dkim_auth_chart.destroy();
        this.dkim_auth_chart = await this.createPieChart("dkim_auth_chart", this.sortedMap(summary.dmarc.dkim_auth_results), resultColorMap);


        if (this.tls_orgs_chart) this.tls_orgs_chart.destroy();
        this.tls_orgs_chart = await this.createPieChart("tls_orgs_chart", this.sortedMap(summary.tls.orgs), orgColorMap, function (label) {
            window.location.hash = "#/tls-reports?org=" + encodeURIComponent(label);
        });

        if (this.tls_domains_chart) this.tls_domains_chart.destroy();
        this.tls_domains_chart = await this.createPieChart("tls_domains_chart", this.sortedMap(summary.tls.domains), null, function (label) {
            window.location.hash = "#/tls-reports?domain=" + encodeURIComponent(label);
        });

        if (this.tls_policy_types) this.tls_policy_types.destroy();
        this.tls_policy_types = await this.createPieChart("tls_policy_types_chart", this.sortedMap(summary.tls.policy_types), null);

        if (this.sts_policy_results) this.sts_policy_results.destroy();
        this.sts_policy_results = await this.createPieChart("sts_policy_results_chart", this.sortedMap(summary.tls.sts_policy_results), resultColorMap);

        if (this.sts_failure_types) this.sts_failure_types.destroy();
        this.sts_failure_types = await this.createPieChart("sts_failure_types_chart", this.sortedMap(summary.tls.sts_failure_types), resultColorMap);

        if (this.tlsa_policy_results) this.tlsa_policy_results.destroy();
        this.tlsa_policy_results = await this.createPieChart("tlsa_policy_results_chart", this.sortedMap(summary.tls.tlsa_policy_results), resultColorMap);

        if (this.tlsa_failure_types) this.tlsa_failure_types.destroy();
        this.tlsa_failure_types = await this.createPieChart("tlsa_failure_types_chart", this.sortedMap(summary.tls.tlsa_failure_types), resultColorMap);
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
        // There is a weird bug that leads to charts sometimes not being rendered.
        // This bad hack seems to avoid it by waiting one cycle...
        await new Promise(r => setTimeout(r, 0));

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
                onClick: function (event, element) {
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
            <style>
                ${(this.classesToHide ?? []).map(c => "." + c).join(", ")} {
                    display: none;
                }
            </style>

            <h1>Dashboard</h1>

            <div class="module stats">
                <span>Mails: <b>${this.mails}</b></span>
                <span>XML Files: <b>${this.xmlFiles}</b></span>
                <span>DMARC Reports: <b>${this.dmarcReports}</b></span>
                <span>JSON Files: <b>${this.jsonFiles}</b></span>
                <span>SMTP TLS Reports: <b>${this.tlsReports}</b></span>
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
                        ${this.filterDomains.map((domain) =>
                        html`<option
                                ?selected=${this.params.domain === encodeURIComponent(domain)}
                                value="${encodeURIComponent(domain)}">${domain}</option>`
                        )}
                    </select>
                </span>
            </div>

            <h2 class="dmarc_charts">DMARC Summary</h2>
            <div class="grid dmarc_charts">
                <div class="module dmarc_orgs">
                    <h3>DMARC Organizations</h3>
                    <canvas class="dmarc_orgs_chart"></canvas>
                </div>

                <div class="module dmarc_domains">
                    <h3>DMARC Domains</h3>
                    <canvas class="dmarc_domains_chart"></canvas>
                </div>

                <div class="module spf_policy">
                    <h3>SPF Policy Results</h3>
                    <canvas class="spf_policy_chart"></canvas>
                </div>

                <div class="module dkim_policy">
                    <h3>DKIM Policy Results</h3>
                    <canvas class="dkim_policy_chart"></canvas>
                </div>

                <div class="module spf_auth">
                    <h3>SPF Auth Results</h3>
                    <canvas class="spf_auth_chart"></canvas>
                </div>

                <div class="module dkim_auth">
                    <h3>DKIM Auth Results</h3>
                    <canvas class="dkim_auth_chart"></canvas>
                </div>
            </div>

            <h2 class="tls_charts">SMTP TLS Report Summary</h2>
            <div class="grid tls_charts">
                <div class="module tls_orgs">
                    <h3>TLS Organizations</h3>
                    <canvas class="tls_orgs_chart"></canvas>
                </div>

                <div class="module tls_domains">
                    <h3>TLS Domains</h3>
                    <canvas class="tls_domains_chart"></canvas>
                </div>

                <div class="module tls_policy_types">
                    <h3>TLS Policy Types</h3>
                    <canvas class="tls_policy_types_chart"></canvas>
                </div>

                <div class="module sts_policy_results">
                    <h3>MTA-STS Policy Results</h3>
                    <canvas class="sts_policy_results_chart"></canvas>
                </div>

                <div class="module sts_failure_types">
                    <h3>MTA-STS Failure Types</h3>
                    <canvas class="sts_failure_types_chart"></canvas>
                </div>

                <div class="module tlsa_policy_results">
                    <h3>DANE TLSA Policy Results</h3>
                    <canvas class="tlsa_policy_results_chart"></canvas>
                </div>

                <div class="module tlsa_failure_types">
                    <h3>DANE TLSA Failure Types</h3>
                    <canvas class="tlsa_failure_types_chart"></canvas>
                </div>
            </div>
        `;
    }
}

customElements.define("drv-dashboard", Dashboard);
