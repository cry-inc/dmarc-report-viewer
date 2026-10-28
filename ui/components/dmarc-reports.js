import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { clearHashQuery, hasFilters, openCustomTimeDialog, setHashQueryParams, timeFilterTooltip } from "../utils.js";

export class DmarcReports extends LitElement {
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
        if (this.params.flagged_dkim === "true" || this.params.flagged_dkim === "false") {
            urlParams.push("flagged_dkim=" + this.params.flagged_dkim);
        }
        if (this.params.flagged_spf === "true" || this.params.flagged_spf === "false") {
            urlParams.push("flagged_spf=" + this.params.flagged_spf);
        }
        if (this.params.flagged_dmarc === "true" || this.params.flagged_dmarc === "false") {
            urlParams.push("flagged_dmarc=" + this.params.flagged_dmarc);
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
        let url = "dmarc-reports";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const response = await fetch(url);
        const data = await response.json();
        this.reports = data.reports;
        this.domains = data.domains;
        this.orgs = data.orgs;
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
        this.domains.sort();
        this.orgs.sort();
    }

    render() {
        return html`
            <h1>DMARC Reports</h1>
            <div>
                Filters:
                <drv-dropdown-select class="ml" title="Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/dmarc-reports", "flagged", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="DKIM Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_dkim) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_dkim === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_dkim === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/dmarc-reports", "flagged_dkim", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="SPF Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_spf) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_spf === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_spf === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/dmarc-reports", "flagged_spf", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="DMARC Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_dmarc) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_dmarc === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_dmarc === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/dmarc-reports", "flagged_dmarc", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Domain"
                    .items="${[
                        { value: "", displayName: "All", selected: !this.params.domain },
                        ...this.domains.map(domain => ({
                            value: domain,
                            displayName: domain,
                            selected: this.params.domain === domain,
                        })),
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/dmarc-reports", "domain", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Org"
                    .items="${[
                        { value: "", displayName: "All", selected: !this.params.org },
                        ...this.orgs.map(org => ({
                            value: org,
                            displayName: org,
                            selected: this.params.org === org,
                        })),
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/dmarc-reports", "org", value)}"></drv-dropdown-select>
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
                    .onChange="${(value) => value === "custom" ? openCustomTimeDialog(this.params.before, this.params.after, "#/dmarc-reports") : setHashQueryParams("#/dmarc-reports", "days", value)}"></drv-dropdown-select>
                <button class="button" ?disabled="${!hasFilters()}" @click="${clearHashQuery}">Clear</button>
            </div>
            <drv-dmarc-report-table .reports="${this.reports}" .filterCallback="${(filter, value) => setHashQueryParams("#/dmarc-reports", filter, value)}"></drv-dmarc-report-table>
        `;
    }
}

customElements.define("drv-dmarc-reports", DmarcReports);
