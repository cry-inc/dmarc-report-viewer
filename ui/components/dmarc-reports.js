import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";
import { decodeParam, navigate, timeQueryParams } from "../utils.js";

export class DmarcReports extends LitElement {
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
        navigate("dmarc-reports", this.params, { ip: encodeURIComponent(event.target.value.trim()) });
    }

    onDnsChange(event) {
        navigate("dmarc-reports", this.params, { dns: encodeURIComponent(event.target.value.trim()) });
    }

    async getDomains() {
        const response = await fetch("summary");
        const summary = await response.json();
        this.domains = Object.keys(summary.dmarc.domains).sort();
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateReports();
        }
    }

    async updateReports() {
        const urlParams = timeQueryParams(this.params);
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
        if (this.params.dns) {
            urlParams.push("dns=" + encodeURIComponent(this.params.dns));
        }
        let url = "dmarc-reports";
        if (urlParams.length > 0) {
            url += "?" + urlParams.join("&");
        }
        const response = await fetch(url);
        this.reports = await response.json();
        this.reports.sort((a, b) => b.date_begin - a.date_begin);
        this.filtered = this.filtered = urlParams.length > 0;
    }

    render() {
        return html`
            <h1>DMARC Reports</h1>
            <div>
                ${this.filtered ?
                    html`Filter active! <a class="ml button" href="#/dmarc-reports">Show all Reports</a>` :
                    html`Filters:
                        <a class="ml button mr-5" href="#/dmarc-reports?flagged=true">Reports with Problems</a>
                        <a class="button mr-5" href="#/dmarc-reports?flagged_dkim=true">Reports with DKIM Problems</a>
                        <a class="button mr-5" href="#/dmarc-reports?flagged_spf=true">Reports with SPF Problems</a>
                        <a class="button" href="#/dmarc-reports?flagged_dmarc=true">Reports with DMARC Problems</a>
                    `
                }
                <drv-domain-filter route="dmarc-reports" .params="${this.params}" .domains="${this.domains}"></drv-domain-filter>
                <label>Time Span:
                    <drv-time-filter route="dmarc-reports" .params="${this.params}"></drv-time-filter>
                </label>
                <label>Source IP:
                    <input type="text" size="25" placeholder="e.g. 192.0.2.1" .value="${decodeParam(this.params.ip)}" @change="${this.onIpChange}">
                </label>
                <label><span class="help" title="Shows reports with a source IP that has a DNS name containing this text. The first search can take a while.">Source IP DNS</span>:
                    <input type="text" size="25" placeholder="e.g. example.com" .value="${decodeParam(this.params.dns)}" @change="${this.onDnsChange}">
                </label>
            </div>
            <drv-dmarc-report-table .reports="${this.reports}"></drv-dmarc-report-table>
        `;
    }
}

customElements.define("drv-dmarc-reports", DmarcReports);
