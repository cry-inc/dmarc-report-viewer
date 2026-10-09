import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { decodeParam, navigate } from "../utils.js";

export class TlsReports extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        reports: { type: Array },
        domains: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.reports = [];
        this.filtered = false;
        this.domains = [];
        this.getDomains();
    }

    onIpChange(event) {
        navigate("tls-reports", this.params, { ip: encodeURIComponent(event.target.value.trim()) });
    }

    async getDomains() {
        const response = await fetch("summary");
        const summary = await response.json();
        this.domains = Object.keys(summary.tls.domains).sort();
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
        let url = "tls-reports";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const response = await fetch(url);
        this.reports = await response.json();
        this.reports.sort((a, b) => new Date(b.date_begin) - new Date(a.date_begin));
        this.filtered = this.filtered = urlParams.length > 0;
    }

    render() {
        return html`
            <h1>SMTP TLS Reports</h1>
            <div>
                ${this.filtered ?
                    html`Filter active! <a class="ml button" href="#/tls-reports">Show all Reports</a>` :
                    html`Filters:
                        <a class="ml button mr-5" href="#/tls-reports?flagged=true">Reports with Problems</a>
                        <a class="button mr-5" href="#/tls-reports?flagged_sts=true">Reports with MTA-STS Problems</a>
                        <a class="button mr-5" href="#/tls-reports?flagged_tlsa=true">Reports with TLSA Problems</a>
                    `
                }
                <drv-domain-filter route="tls-reports" .params="${this.params}" .domains="${this.domains}"></drv-domain-filter>
                <label>Source IP:
                    <input type="text" size="25" placeholder="e.g. 192.0.2.1" .value="${decodeParam(this.params.ip)}" @change="${this.onIpChange}">
                </label>
            </div>
            <drv-tls-report-table .reports="${this.reports}"></drv-tls-report-table>
        `;
    }
}

customElements.define("drv-tls-reports", TlsReports);
