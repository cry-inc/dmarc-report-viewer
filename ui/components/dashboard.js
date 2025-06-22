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
        tlsrptReports: { type: Number },
        lastUpdate: { type: Number },
        dmarcDomains: { type: Array },
        tlsrptDomains: { type: Array },
        classesToHide: { type: Array },
    };

    constructor() {
        super();

        this.params = {};
        this.mails = 0;
        this.xmlFiles = 0;
        this.jsonFiles = 0;
        this.dmarcReports = 0;
        this.tlsrptReports = 0;
        this.lastUpdate = 0;
        this.dmarcDomains = [];
        this.tlsrptDomains = [];
        this.filterDomains = [];

        this.getDomains();
    }

    async getDomains() {
        const response = await fetch("summary");
        const summary = await response.json();
        this.dmarcDomains = Object.keys(summary.dmarc.domains);
        this.dmarcDomains.sort();
        this.tlsrptDomains = Object.keys(summary.tlsrpt.domains);
        this.tlsrptDomains.sort();
        this.filterDomains = [...new Set([
            ...this.dmarcDomains,
            ...this.tlsrptDomains
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
        this.jsonFiles = summary.tlsrpt.files;
        this.dmarcReports = summary.dmarc.reports;
        this.tlsrptReports = summary.tlsrpt.reports;
        this.lastUpdate = summary.last_update;

        this.classesToHide = [];
        if (this.dmarcReports === 0) this.classesToHide.push("dmarc");
        if (this.tlsrptReports === 0) this.classesToHide.push("tlsrpt");
        if (this.dmarcReports > 0) this.classesToHide.push("no_dmarc_reports");
        if (this.tlsrptReports > 0) this.classesToHide.push("no_tlsrpt_reports");
        if (Object.values(summary.dmarc.orgs).every((v) => v === 0)) this.classesToHide.push("dmarc_orgs");
        if (Object.values(summary.dmarc.domains).every((v) => v === 0)) this.classesToHide.push("dmarc_domains");
        if (Object.values(summary.dmarc.spf_policy_results).every((v) => v === 0)) this.classesToHide.push("spf_policy");
        if (Object.values(summary.dmarc.dkim_policy_results).every((v) => v === 0)) this.classesToHide.push("dkim_policy");
        if (Object.values(summary.dmarc.spf_auth_results).every((v) => v === 0)) this.classesToHide.push("spf_auth");
        if (Object.values(summary.dmarc.dkim_auth_results).every((v) => v === 0)) this.classesToHide.push("dkim_auth");
        if (Object.values(summary.tlsrpt.orgs).every((v) => v === 0)) this.classesToHide.push("tlsrpt_orgs");
        if (Object.values(summary.tlsrpt.domains).every((v) => v === 0)) this.classesToHide.push("tlsrpt_domains");
        if (Object.values(summary.tlsrpt.policy_types).every((v) => v === 0)) this.classesToHide.push("tlsrpt_policy_types");
        if (Object.values(summary.tlsrpt.sts_policy_results).every((v) => v === 0)) this.classesToHide.push("sts_policy_results");
        if (Object.values(summary.tlsrpt.sts_failure_types).every((v) => v === 0)) this.classesToHide.push("sts_failure_types");
        if (Object.values(summary.tlsrpt.tlsa_policy_results).every((v) => v === 0)) this.classesToHide.push("tlsa_policy_results");
        if (Object.values(summary.tlsrpt.tlsa_failure_types).every((v) => v === 0)) this.classesToHide.push("tlsa_failure_types");
        
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


        if (this.tlsrpt_orgs_chart) this.tlsrpt_orgs_chart.destroy();
        this.tlsrpt_orgs_chart = await this.createPieChart("tlsrpt_orgs_chart", this.sortedMap(summary.tlsrpt.orgs), orgColorMap, function (label) {
            window.location.hash = "#/tlsrpt-reports?org=" + encodeURIComponent(label);
        });

        if (this.tlsrpt_domains_chart) this.tlsrpt_domains_chart.destroy();
        this.tlsrpt_domains_chart = await this.createPieChart("tlsrpt_domains_chart", this.sortedMap(summary.tlsrpt.domains), null, function (label) {
            window.location.hash = "#/tlsrpt-reports?domain=" + encodeURIComponent(label);
        });

        if (this.tlsrpt_policy_types) this.tlsrpt_policy_types.destroy();
        this.tlsrpt_policy_types = await this.createPieChart("tlsrpt_policy_types_chart", this.sortedMap(summary.tlsrpt.policy_types), null);

        if (this.sts_policy_results) this.sts_policy_results.destroy();
        this.sts_policy_results = await this.createPieChart("sts_policy_results_chart", this.sortedMap(summary.tlsrpt.sts_policy_results), resultColorMap);

        if (this.sts_failure_types) this.sts_failure_types.destroy();
        this.sts_failure_types = await this.createPieChart("sts_failure_types_chart", this.sortedMap(summary.tlsrpt.sts_failure_types), resultColorMap);

        if (this.tlsa_policy_results) this.tlsa_policy_results.destroy();
        this.tlsa_policy_results = await this.createPieChart("tlsa_policy_results_chart", this.sortedMap(summary.tlsrpt.tlsa_policy_results), resultColorMap);

        if (this.tlsa_failure_types) this.tlsa_failure_types.destroy();
        this.tlsa_failure_types = await this.createPieChart("tlsa_failure_types_chart", this.sortedMap(summary.tlsrpt.tlsa_failure_types), resultColorMap);
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
            <style>
                ${(this.classesToHide ?? []).map(c => `.${c}`).join(",\n")} {
                    display: none;
                }
            </style>

            <h1>Dashboard</h1>

            <div class="module stats">
                <span>Mails: <b>${this.mails}</b></span>
                <span class="dmarc">XML Files: <b>${this.xmlFiles}</b></span>
                <span class="dmarc">DMARC Reports: <b>${this.dmarcReports}</b></span>
                <span class="tlsrpt">JSON Files: <b>${this.jsonFiles}</b></span>
                <span class="tlsrpt">SMTP TLS Reports: <b>${this.tlsrptReports}</b></span>
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

            <h2>DMARC Summary</h2>
            <p class="no_dmarc_reports">No DMARC reports found.</p>

            <div class="grid dmarc">
                <div class="module dmarc dmarc_orgs">
                    <h3>DMARC Organizations</h3>
                    <canvas class="dmarc_orgs_chart"></canvas>
                </div>

                <div class="module dmarc dmarc_domains">
                    <h3>DMARC Domains</h3>
                    <canvas class="dmarc_domains_chart"></canvas>
                </div>

                <div class="module dmarc spf_policy">
                    <h3>SPF Policy Results</h3>
                    <canvas class="spf_policy_chart"></canvas>
                </div>

                <div class="module dmarc dkim_policy">
                    <h3>DKIM Policy Results</h3>
                    <canvas class="dkim_policy_chart"></canvas>
                </div>

                <div class="module dmarc spf_auth">
                    <h3>SPF Auth Results</h3>
                    <canvas class="spf_auth_chart"></canvas>
                </div>

                <div class="module dmarc dkim_auth">
                    <h3>DKIM Auth Results</h3>
                    <canvas class="dkim_auth_chart"></canvas>
                </div>
            </div>

            <h2>SMTP TLS Report Summary</h2>
            <p class="no_tlsrpt_reports">No SMTP TLS reports found.</p>

            <div class="grid tlsrpt">
                <div class="module tlsrpt tlsrpt_orgs">
                    <h3>TLS Organizations</h3>
                    <canvas class="tlsrpt_orgs_chart"></canvas>
                </div>

                <div class="module tlsrpt tlsrpt_domains">
                    <h3>TLS Domains</h3>
                    <canvas class="tlsrpt_domains_chart"></canvas>
                </div>

                <div class="module tlsrpt tlsrpt_policy_types">
                    <h3>TLS Policy Types</h3>
                    <canvas class="tlsrpt_policy_types_chart"></canvas>
                </div>

                <div class="module tlsrpt sts_policy_results">
                    <h3>MTA-STS Policy Results</h3>
                    <canvas class="sts_policy_results_chart"></canvas>
                </div>

                <div class="module tlsrpt sts_failure_types">
                    <h3>MTA-STS Failure Types</h3>
                    <canvas class="sts_failure_types_chart"></canvas>
                </div>

                <div class="module tlsrpt tlsa_policy_results">
                    <h3>DANE TLSA Policy Results</h3>
                    <canvas class="tlsa_policy_results_chart"></canvas>
                </div>

                <div class="module tlsrpt tlsa_failure_types">
                    <h3>DANE TLSA Failure Types</h3>
                    <canvas class="tlsa_failure_types_chart"></canvas>
                </div>
            </div>
        `;
    }
}

customElements.define("drv-dashboard", Dashboard);
