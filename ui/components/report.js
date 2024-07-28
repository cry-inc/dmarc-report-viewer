import { LitElement, html, css } from "lit";

export class Report extends LitElement {
    static styles = css`
        table {
            width: 100%;
            margin-bottom: 20px;
        }
    
        th {
            text-align: left;
            background-color: #efefef;
            width: 200px;
        }

        td, th {
            padding-left: 10px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
        }

        .na {
            color: #ccc;
        }

        .bigHeader {
            font-size: 20px;
        }

        .result {
            border-radius: 3px;
            padding-left: 4px;
            padding-right: 4px;
            background-color: #888;
            color: white;
        }

        .result.negative {
            background-color: #f00;
        }

        .result.positive {
            background-color: #090;
        }
    `;

    static get properties() {
        return {
            id: { type: String },
            report: { type: Object, attribute: false }
        };
    }

    constructor() {
        super();
        this.id = null;
        this.report = null;
    }

    async updated(changedProperties) {
        if (changedProperties.has("id") && changedProperties.id !== this.id && this.id) {
            const response = await fetch("reports/" + this.id);
            this.report = await response.json();
        }
    }

    renderOptional(value) {
        if (value !== null && value !== undefined) {
            return html`${value}`;
        } else {
            return html`<span class="na">n/a</span>`;
        }
    }

    renderResultBadge(result) {
        if (result === "fail" || result === "temperror" ||
            result === "permerror" || result === "softfail" ||
            result === "quarantine" || result === "reject"
        ) {
            return html`<span class="result negative">${result}</span>`;
        } else if (result === "pass") {
            return html`<span class="result positive">${result}</span>`;
        } else if (result !== null || result !== undefined) {
            return html`<span class="na">n/a</span>`;
        } else {
            return html`<span class="result neutral">${result}</span>`;
        }
    }

    render() {
        if (!this.report) {
            return html`No report loaded`;
        }

        let errors = null;
        if (this.report.report_metadata.error) {
            errors = this.report.report_metadata.error.join(", ");
        }

        return html`
            <table>
                <tr>
                    <th colspan="2" class="bigHeader">Report</th>
                </tr>
                <tr>
                    <th>Id</th>
                    <td>${this.report.report_metadata.report_id}</td>
                </tr>
                <tr>
                    <th>Org</th>
                    <td>${this.report.report_metadata.org_name}</td>
                </tr>
                <tr>
                    <th>Records</th>
                    <td>${this.report.record.length}</td>
                </tr>
                <tr>
                    <th>Date Range Begin</th>
                    <td>${new Date(this.report.report_metadata.date_range.begin * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <th>Date Range End</th>
                    <td>${new Date(this.report.report_metadata.date_range.end * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <th>E-Mail</th>
                    <td>${this.report.report_metadata.email}</td>
                </tr>
                <tr>
                    <th>Extra Contact Info</th>
                    <td>${this.renderOptional(this.report.report_metadata.extra_contact_info)}</td>
                </tr>
                <tr>
                    <th>Errors</th>
                    <td>${this.renderOptional(errors)}</td>
                </tr>
                <tr>
                    <th>Version</th>
                    <td>${this.renderOptional(this.report.version)}</td>
                </tr>
                <tr>
                    <th colspan="2">Published Policy</th>
                </tr>
                <tr>
                    <th>Domain</th>
                    <td>${this.report.policy_published.domain}</td>
                </tr>
                <tr>
                    <th>adkim</th>
                    <td>${this.renderOptional(this.report.policy_published.adkim)}</td>
                </tr>
                <tr>
                    <th>aspf</th>
                    <td>${this.renderOptional(this.report.policy_published.aspf)}</td>
                </tr>
                <tr>
                    <th>p</th>
                    <td>${this.report.policy_published.p}</td>
                </tr>
                <tr>
                    <th>sp</th>
                    <td>${this.renderOptional(this.report.policy_published.sp)}</td>
                </tr>
                <tr>
                    <th>pct</th>
                    <td>${this.report.policy_published.pct}</td>
                </tr>
                <tr>
                    <th>fo</th>
                    <td>${this.renderOptional(this.report.policy_published.fo)}</td>
                </tr>
                ${this.report.record.map((record) => html`
                    <tr>
                        <td colspan="2">&nbsp;</td>
                    </tr>
                    <tr>
                        <th colspan="2" class="bigHeader">Record</th>
                    </tr>
                    <tr>
                        <th>Source IP</th>
                        <td>${record.row.source_ip}</td>
                    </tr>
                    <tr>
                        <th>Count</th>
                        <td>${record.row.count}</td>
                    </tr>
                    <tr>
                        <th>Policy Disposition</th>
                        <td>${this.renderResultBadge(record.row.policy_evaluated.disposition)}</td>
                    </tr>
                    <tr>
                        <th>Policy DKIM</th>
                        <td>${this.renderResultBadge(record.row.policy_evaluated.dkim)}</td>
                    </tr>
                    <tr>
                        <th>Policy SPF</th>
                        <td>${this.renderResultBadge(record.row.policy_evaluated.spf)}</td>
                    </tr>
                    <tr>
                        <th>Policy Reason</th>
                        <td>
                            ${record.row.policy_evaluated.reason ?
                                record.row.policy_evaluated.reason.map(
                                    (reason) => html`${reason.kind} ${reason.comment}`
                                ) : html`<span class="na">n/a</span>`
                            }
                        </td>
                    </tr>
                    <tr>
                        <th>Header From</th>
                        <td>${record.identifiers.header_from}</td>
                    </tr>
                    <tr>
                        <th>Envelope From</th>
                        <td>${this.renderOptional(record.identifiers.envelope_from)}</td>
                    </tr>
                    <tr>
                        <th>Envelope To</th>
                        <td>${this.renderOptional(record.identifiers.envelope_to)}</td>
                    </tr>
                    ${record.auth_results.spf.map((result) => html`
                        <tr>
                            <th colspan="2">SPF Auth Result</th>
                        </tr>
                        <tr>
                            <th>Domain</th>
                            <td>${result.domain}</td>
                        </tr>
                        <tr>
                            <th>Scope</th>
                            <td>${this.renderOptional(result.scope)}</td>
                        </tr>
                        <tr>
                            <th>Result</th>
                            <td>${this.renderResultBadge(result.result)}</td>
                        </tr>
                    `)}
                    ${(record.auth_results.dkim ?
                        record.auth_results.dkim : []).map((result) => html`
                        <tr>
                            <th colspan="2">DKIM Auth Result</th>
                        </tr>
                        <tr>
                            <th>Domain</th>
                            <td>${result.domain}</td>
                        </tr>
                        <tr>
                            <th>Scope</th>
                            <td>${this.renderOptional(result.selector)}</td>
                        </tr>
                        <tr>
                            <th>Result</th>
                            <td>${this.renderResultBadge(result.result)}</td>
                        </tr>
                        <tr>
                            <th>Human Result</th>
                            <td>${this.renderOptional(result.human_result)}</td>
                        </tr>
                    `)}
                `)}
            </table>
        `;
    }
}

customElements.define("dmarc-report", Report);
