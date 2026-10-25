import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { clearHashQuery, hasFilters, openCustomTimeDialog, setHashQueryParams, timeFilterTooltip } from "../utils.js";

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
                    .onChange="${(value) => setHashQueryParams("#/mails", "oversized", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Type"
                    .items="${[
                        { value: "", displayName: "All", selected: !this.params.attachment },
                        { value: "dmarc", displayName: "DMARC", selected: this.params.attachment === "dmarc" },
                        { value: "tls", displayName: "TLS", selected: this.params.attachment === "tls" },
                        { value: "none", displayName: "None", selected: this.params.attachment === "none" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/mails", "attachment", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Duplicates"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.duplicates) },
                        { value: "true", displayName: "Yes", selected: this.params.duplicates === "true" },
                        { value: "false", displayName: "No", selected: this.params.duplicates === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/mails", "duplicates", value)}"></drv-dropdown-select>
                <drv-dropdown-select title="Errors"
                    .items="${[
                        { value: "", displayName: "All", selected: !["true", "false"].includes(this.params.errors) },
                        { value: "true", displayName: "Yes", selected: this.params.errors === "true" },
                        { value: "false", displayName: "No", selected: this.params.errors === "false" },
                    ]}"
                    .onChange="${(value) => setHashQueryParams("#/mails", "errors", value)}"></drv-dropdown-select>
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
                    .onChange="${(value) => value === "custom" ? openCustomTimeDialog(this.params.before, this.params.after, "#/mails") : setHashQueryParams("#/mails", "days", value)}"></drv-dropdown-select>
                <button class="button" ?disabled="${!hasFilters()}" @click="${clearHashQuery}">Clear</button>
            </div>
            <drv-mail-table .mails="${this.mails}"></drv-mail-table>
        `;
    }
}

customElements.define("drv-mails", Mails);
