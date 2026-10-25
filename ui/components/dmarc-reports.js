import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { timeRangeDialog } from "./time-range.js";

export class DmarcReports extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        reports: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.reports = [];
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateReports();
        }
    }

    setFilter(name, value) {
        const hashParams = document.location.hash.split("?")[1] || "";
        const params = new URLSearchParams(hashParams);
        if (value) {
            params.set(name, value);
        } else {
            params.delete(name);
        }
        if (name === "days") {
            params.delete("before");
            params.delete("after");
        }
        const query = params.toString();
        document.location.hash = query ? `#/dmarc-reports?${query}` : "#/dmarc-reports";
    }

    clearFilters() {
        document.location.hash = "#/dmarc-reports";
    }

    async openCustomDateDialog() {
        const selected = await timeRangeDialog(
            this.params.before ? new Date(Number(this.params.before) * 1000) : undefined,
            this.params.after ? new Date(Number(this.params.after) * 1000) : undefined,
        );
        if (!selected) return false;

        const hashParams = document.location.hash.split("?")[1] || "";
        const params = new URLSearchParams(hashParams);
        params.delete("days");

        for (const [name, date] of [["before", selected.before], ["after", selected.after]]) {
            if (date) {
                params.set(name, Math.floor(date.getTime() / 1000).toString());
            } else {
                params.delete(name);
            }
        }

        const query = params.toString();
        document.location.hash = query ? `#/dmarc-reports?${query}` : "#/dmarc-reports";
    }

    timeFilterTooltip() {
        const excludedBefore = this.params.after ? new Date(Number(this.params.after) * 1000).toLocaleString() : undefined;
        const excludedAfter = this.params.before ? new Date(Number(this.params.before) * 1000).toLocaleString() : undefined;
        if (excludedBefore && excludedAfter) return `All reports before ${excludedBefore} and after ${excludedAfter} are excluded.`;
        if (excludedBefore) return `All reports before ${excludedBefore} are excluded.`;
        if (excludedAfter) return `All reports after ${excludedAfter} are excluded.`;
        return "";
    }

    hasFilters() {
        return Boolean(document.location.hash.split("?")[1]);
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
        this.reports = await response.json();
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
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
                    .onChange="${(value) => this.setFilter("flagged", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="DKIM Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_dkim) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_dkim === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_dkim === "false" },
                    ]}"
                    .onChange="${(value) => this.setFilter("flagged_dkim", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="SPF Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_spf) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_spf === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_spf === "false" },
                    ]}"
                    .onChange="${(value) => this.setFilter("flagged_spf", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="DMARC Problems"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.flagged_dmarc) },
                        { value: "true", displayName: "Yes", selected: this.params.flagged_dmarc === "true" },
                        { value: "false", displayName: "No", selected: this.params.flagged_dmarc === "false" },
                    ]}"
                    .onChange="${(value) => this.setFilter("flagged_dmarc", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Time" .tooltip="${this.timeFilterTooltip()}"
                    .items="${[
                        { value: "", displayName: "All", selected: ![3, 7, 30, 180, 365].includes(Number(this.params.days)) && !(this.params.before || this.params.after) },
                        { value: "3", displayName: "Last 3 days", selected: this.params.days === "3" },
                        { value: "7", displayName: "Last week", selected: this.params.days === "7" },
                        { value: "30", displayName: "Last month", selected: this.params.days === "30" },
                        { value: "180", displayName: "Last 6 months", selected: this.params.days === "180" },
                        { value: "365", displayName: "Last year", selected: this.params.days === "365" },
                        { value: "custom", displayName: "Custom...", selected: Boolean(this.params.before || this.params.after) },
                    ]}"
                    .onChange="${(value) => value === "custom" ? this.openCustomDateDialog() : this.setFilter("days", value)}"></drv-dropdown-select>
                <button class="button" ?disabled="${!this.hasFilters()}" @click="${this.clearFilters}">Clear</button>
            </div>
            <drv-dmarc-report-table .reports="${this.reports}"></drv-dmarc-report-table>
        `;
    }
}

customElements.define("drv-dmarc-reports", DmarcReports);
