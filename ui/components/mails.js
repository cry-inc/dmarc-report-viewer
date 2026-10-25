import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { timeRangeDialog } from "./time-range.js";

export class Mails extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        mails: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.mails = [];
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateMails();
        }
    }

    async updateMails() {
        const queryParams = [];
        if (this.params.oversized === "true" || this.params.oversized === "false") {
            queryParams.push("oversized=" + this.params.oversized);
        }
        if (this.params.sender) {
            queryParams.push("sender=" + encodeURIComponent(this.params.sender));
        }
        if (this.params.attachment) {
            queryParams.push("attachment=" + encodeURIComponent(this.params.attachment));
        }
        if (this.params.errors === "true" || this.params.errors === "false") {
            queryParams.push("errors=" + this.params.errors);
        }
        if (this.params.duplicates === "true" || this.params.duplicates === "false") {
            queryParams.push("duplicates=" + this.params.duplicates);
        }
        if (this.params.days) {
            const days = Number(this.params.days);
            const after = Math.floor(Date.now() / 1000) - days * 24 * 60 * 60;
            queryParams.push("after=" + after);
        }
        if (this.params.after) {
            queryParams.push("after=" + this.params.after);
        }
        if (this.params.before) {
            queryParams.push("before=" + this.params.before);
        }
        let url = "mails";
        if (queryParams.length > 0) {
            url += "?" + queryParams.join("&");
        }
        const mailsResponse = await fetch(url);
        this.mails = await mailsResponse.json();
        this.mails.sort((a, b) => b.date - a.date);
    }

    setFilter(name, value) {
        // Get filter params from hash part of the URL and parse them
        const hashParams = document.location.hash.split("?")[1] || "";
        const params = new URLSearchParams(hashParams);

        // Replace/add/delete value
        if (value) {
            params.set(name, value);
        } else {
            params.delete(name);
        }

        // Prevent incompatible combinations
        if (name === "days") {
            params.delete("before");
            params.delete("after");
        }

        // Build new string and update hash
        const query = params.toString();
        document.location.hash = query ? `#/mails?${query}` : "#/mails";
    }

    clearFilters() {
        document.location.hash = "#/mails";
    }

    async openCustomDateDialog() {
        const selected = await timeRangeDialog(
            this.params.before ? new Date(Number(this.params.before) * 1000) : undefined,
            this.params.after ? new Date(Number(this.params.after) * 1000) : undefined,
        );

        // Keep the current time filter when the dialog is cancelled
        if (!selected) return false;

        const hashParams = document.location.hash.split("?")[1] || "";
        const params = new URLSearchParams(hashParams);
        params.delete("days"); // Prevent incompatible combinations

        for (const [name, date] of [["before", selected.before], ["after", selected.after]]) {
            if (date) {
                params.set(name, Math.floor(date.getTime() / 1000).toString());
            } else {
                params.delete(name);
            }
        }

        // Update hash query params
        const query = params.toString();
        document.location.hash = query ? `#/mails?${query}` : "#/mails";
    }

    hasFilters() {
        const hashParams = document.location.hash.split("?")[1];
        return Boolean(hashParams);
    }

    timeFilterTooltip() {
        const excludedBefore = this.params.after ? new Date(Number(this.params.after) * 1000).toLocaleString() : undefined;
        const excludedAfter = this.params.before ? new Date(Number(this.params.before) * 1000).toLocaleString() : undefined;
        if (excludedBefore && excludedAfter) return `All mails before ${excludedBefore} and after ${excludedAfter} are excluded.`;
        if (excludedBefore) return `All mails before ${excludedBefore} are excluded.`;
        if (excludedAfter) return `All mails after ${excludedAfter} are excluded.`;
        return "";
    }

    render() {
        return html`
            <h1>Mails</h1>
            <div>
                Filters:
                <drv-dropdown-select class="ml" title="Size"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.oversized) },
                        { value: "false", displayName: "Valid", selected: this.params.oversized === "false" },
                        { value: "true", displayName: "Oversized", selected: this.params.oversized === "true" },
                    ]}"
                    .onChange="${(value) => this.setFilter("oversized", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Type"
                    .items="${[
                        { value: "", displayName: "All", selected: !this.params.attachment },
                        { value: "dmarc", displayName: "DMARC", selected: this.params.attachment === "dmarc" },
                        { value: "tls", displayName: "TLS", selected: this.params.attachment === "tls" },
                        { value: "none", displayName: "None", selected: this.params.attachment === "none" },
                    ]}"
                    .onChange="${(value) => this.setFilter("attachment", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Duplicates"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.duplicates) },
                        { value: "true", displayName: "Yes", selected: this.params.duplicates === "true" },
                        { value: "false", displayName: "No", selected: this.params.duplicates === "false" },
                    ]}"
                    .onChange="${(value) => this.setFilter("duplicates", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Errors"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.errors) },
                        { value: "true", displayName: "Yes", selected: this.params.errors === "true" },
                        { value: "false", displayName: "No", selected: this.params.errors === "false" },
                    ]}"
                    .onChange="${(value) => this.setFilter("errors", value)}"></drv-dropdown-select>
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
            <drv-mail-table .mails="${this.mails}"></drv-mail-table>
        `;
    }
}

customElements.define("drv-mails", Mails);
