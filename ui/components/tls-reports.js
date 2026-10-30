import { LitElement, html, nothing } from "lit";
import { globalStyle } from "../style.js";
import { clearHashQuery, downloadFile, hasFilters, openCustomTimeDialog, setHashQueryParams, timeFilterTooltip } from "../utils.js";

export class TlsReports extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        reports: { type: Array },
        domains: { type: Array },
        orgs: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.reports = [];
        this.domains = [];
        this.orgs = [];
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateReports();
        }
    }

    async updateReports() {
        const urlParams = [];
        if (this.params.flagged === "true" || this.params.flagged === "false") {
            urlParams.push("flagged=" + this.params.flagged);
        }
        if (this.params.flagged_sts === "true" || this.params.flagged_sts === "false") {
            urlParams.push("flagged_sts=" + this.params.flagged_sts);
        }
        if (this.params.flagged_tlsa === "true" || this.params.flagged_tlsa === "false") {
            urlParams.push("flagged_tlsa=" + this.params.flagged_tlsa);
        }
        if (this.params.domain) {
            urlParams.push("domain=" + encodeURIComponent(this.params.domain));
        }
        if (this.params.org) {
            urlParams.push("org=" + encodeURIComponent(this.params.org));
        }
        if (this.params.ip) {
            urlParams.push("ip=" + encodeURIComponent(this.params.ip));
        }
        if (this.params.days) {
            const days = Number(this.params.days);
            const after = Math.floor(Date.now() / 1000) - days * 24 * 60 * 60;
            urlParams.push("after=" + after);
        }
        if (this.params.after) {
            urlParams.push("after=" + this.params.after);
        }
        if (this.params.before) {
            urlParams.push("before=" + this.params.before);
        }
        let url = "tls-reports";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const response = await fetch(url);
        const data = await response.json();
        this.reports = data.reports;
        this.domains = data.domains;
        this.orgs = data.orgs;
        this.reports.sort((a, b) => new Date(b.date_begin) - new Date(a.date_begin));
        this.domains.sort();
        this.orgs.sort();
    }

    async exportReports() {
        const reports = await Promise.all(this.reports.map(async (report) => {
            const response = await fetch(`tls-reports/${encodeURIComponent(report.hash)}/json`);
            if (!response.ok) {
                throw new Error(`Failed to export report ${report.hash}: ${response.status}`);
            }
            return response.json();
        }));

        downloadFile("tls-reports.json", JSON.stringify(reports, null, 2), "application/json");
    }

    render() {
        return html`
            <h1>SMTP TLS Reports</h1>
            <div>
                Filters:
                <drv-dropdown-select class="ml" title="Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/tls-reports", "flagged", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="MTA-STS Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_sts) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_sts === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_sts === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/tls-reports", "flagged_sts", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="TLSA Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_tlsa) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_tlsa === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_tlsa === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/tls-reports", "flagged_tlsa", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Domain"
                    .items="${[
                        { value: "", displayName: "All", selected: !this.params.domain },
                        ...this.domains.map(domain => ({
                            value: domain,
                            displayName: domain,
                            selected: this.params.domain === domain,
                        })),
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/tls-reports", "domain", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Org"
                    .items="${[
                        { value: "", displayName: "All", selected: !this.params.org },
                        ...this.orgs.map(org => ({
                            value: org,
                            displayName: org,
                            selected: this.params.org === org,
                        })),
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/tls-reports", "org", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Time" .tooltip="${timeFilterTooltip(this.params.after, this.params.before)}"
                    .items="${[
                        { value: "", displayName: "All", selected: ![3, 7, 30, 180, 365].includes(Number(this.params.days)) && !(this.params.before || this.params.after) },
                        { value: "3", displayName: "Last 3 days", selected: this.params.days === "3" },
                        { value: "7", displayName: "Last week", selected: this.params.days === "7" },
                        { value: "30", displayName: "Last month", selected: this.params.days === "30" },
                        { value: "180", displayName: "Last 6 months", selected: this.params.days === "180" },
                        { value: "365", displayName: "Last year", selected: this.params.days === "365" },
                        { value: "custom", displayName: "Custom...", selected: Boolean(this.params.before || this.params.after) },
                    ]}"
                    .onChange="${(value) => value === "custom" ? openCustomTimeDialog(this.params.before, this.params.after, "#/tls-reports") : setHashQueryParams("#/tls-reports", "days", value)}"></drv-dropdown-select>
                ${this.params.ip ? html`
                    <drv-dropdown-select title="IP"
                        .items="${[
                            { value: "", displayName: "All", selected: false },
                            { value: this.params.ip, displayName: this.params.ip, selected: true },
                        ]}"
                        .onChange="${(value) => setHashQueryParams("#/tls-reports", "ip", value)}"></drv-dropdown-select>
                ` : nothing}
                <button class="button" ?disabled="${!hasFilters()}" @click="${clearHashQuery}">Clear</button>
                <button class="button" title="Export all currently listed reports as JSON file" @click="${this.exportReports}">⬇️</button>
            </div>
            <drv-tls-report-table .reports="${this.reports}" .filterCallback="${(filter, value) => setHashQueryParams("#/tls-reports", filter, value)}"></drv-tls-report-table>
        `;
    }
}

customElements.define("drv-tls-reports", TlsReports);
