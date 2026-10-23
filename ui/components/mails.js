import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { customDateRange } from "../utils.js";

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
        const selected = await customDateRange(
            this.params.before ? new Date(Number(this.params.before) * 1000) : undefined,
            this.params.after ? new Date(Number(this.params.after) * 1000) : undefined,
        );

        // Date dialog was saved without confirmation
        if (!selected) return;

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

    render() {
        return html`
            <h1>Mails</h1>
            <div>
                Size:
                <select .value="${this.params.oversized === "true" ? "true" : this.params.oversized === "false" ? "false" : ""}"
                    @change="${(event) => this.setFilter("oversized", event.target.value)}">
                    <option value="">All</option>
                    <option value="false">Valid</option>
                    <option value="true">Oversized</option>
                </select>
                Type:
                <select .value="${this.params.attachment || ""}"
                    @change="${(event) => this.setFilter("attachment", event.target.value)}">
                    <option value="">All</option>
                    <option value="dmarc">DMARC</option>
                    <option value="tls">TLS</option>
                    <option value="none">None</option>
                </select>
                Duplicates:
                <select .value="${this.params.duplicates === "true" ? "true" : this.params.duplicates === "false" ? "false" : ""}"
                    @change="${(event) => this.setFilter("duplicates", event.target.value)}">
                    <option value="">All</option>
                    <option value="true">Duplicates</option>
                    <option value="false">No Duplicates</option>
                </select>
                Parsing Errors:
                <select .value="${this.params.errors === "true" ? "true" : this.params.errors === "false" ? "false" : ""}"
                    @change="${(event) => this.setFilter("errors", event.target.value)}">
                    <option value="">All</option>
                    <option value="true">Errors</option>
                    <option value="false">No Errors</option>
                </select>
                Time:
                <select .value="${[3, 7, 30, 180, 365].includes(Number(this.params.days)) ? this.params.days : this.params.before || this.params.after ? "custom" : ""}"
                    @change="${(event) => event.target.value === "custom" ? this.openCustomDateDialog() : this.setFilter("days", event.target.value)}">
                    <option value="">All</option>
                    <option value="3">Last 3 days</option>
                    <option value="7">Last week</option>
                    <option value="30">Last month</option>
                    <option value="180">Last six months</option>
                    <option value="365">Last year</option>
                    <option value="custom">Custom...</option>
                </select>
                ${this.params.before || this.params.after ? html`
                    <button @click="${this.openCustomDateDialog}">Edit custom range</button>
                ` : ""}
                <button ?disabled="${!this.hasFilters()}" @click="${this.clearFilters}">Clear filters</button>
            </div>
            <drv-mail-table .mails="${this.mails}"></drv-mail-table>
        `;
    }
}

customElements.define("drv-mails", Mails);
