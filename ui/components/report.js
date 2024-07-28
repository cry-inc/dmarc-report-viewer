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
            padding-left: 5px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
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
            //console.log("Report", this.report);
        }
    }

    render() {
        if (!this.report) {
            return html`No report loaded`;
        }

        let errors = "";
        if (this.report.report_metadata.error) {
            errors = this.report.report_metadata.error.join(", ");
        }

        return html`
            <table>
                <tr>
                    <th colspan="2">Report</th>
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
                    <td>${this.report.report_metadata.extra_contact_info}</td>
                </tr>
                <tr>
                    <th>Errors</th>
                    <td>${errors}</td>
                </tr>
                <tr>
                    <th>Version</th>
                    <td>${this.report.version}</td>
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
                    <td>${this.report.policy_published.adkim}</td>
                </tr>
                <tr>
                    <th>aspf</th>
                    <td>${this.report.policy_published.aspf}</td>
                </tr>
                <tr>
                    <th>p</th>
                    <td>${this.report.policy_published.p}</td>
                </tr>
                <tr>
                    <th>sp</th>
                    <td>${this.report.policy_published.sp}</td>
                </tr>
                <tr>
                    <th>pct</th>
                    <td>${this.report.policy_published.pct}</td>
                </tr>
                <tr>
                    <th>fo</th>
                    <td>${this.report.policy_published.fo}</td>
                </tr>
                ${this.report.record.map((record) => html`
                    <tr>
                        <td colspan="2">&nbsp;</td>
                    </tr>
                    <tr>
                        <th colspan="2">Record</th>
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
                        <td>${record.row.policy_evaluated.disposition}</td>
                    </tr>
                    <tr>
                        <th>Policy DKIM</th>
                        <td>${record.row.policy_evaluated.dkim}</td>
                    </tr>
                    <tr>
                        <th>Policy SPF</th>
                        <td>${record.row.policy_evaluated.spf}</td>
                    </tr>
                    <tr>
                        <th>Header From</th>
                        <td>${record.identifiers.header_from}</td>
                    </tr>
                    <tr>
                        <th>Envelope From</th>
                        <td>${record.identifiers.envelope_from}</td>
                    </tr>
                    <tr>
                        <th>Envelope To</th>
                        <td>${record.identifiers.envelope_from}</td>
                    </tr>
                    <tr>
                        <th colspan="2">SPF Auth Results</th>
                    </tr>
                    ${record.auth_results.spf.map((result) => html`
                        <tr>
                            <th>Domain</th>
                            <td>${result.domain}</td>
                        </tr>
                        <tr>
                            <th>Scope</th>
                            <td>${result.scope}</td>
                        </tr>
                        <tr>
                            <th>Result</th>
                            <td>${result.result}</td>
                        </tr>
                    `)}
                    <tr>
                        <th colspan="2">DKIM Auth Results</th>
                    </tr>
                    ${(record.auth_results.dkim ? record.auth_results.dkim : []).map((result) => html`
                        <tr>
                            <th>Domain</th>
                            <td>${result.domain}</td>
                        </tr>
                        <tr>
                            <th>Scope</th>
                            <td>${result.selector}</td>
                        </tr>
                        <tr>
                            <th>Result</th>
                            <td>${result.result}</td>
                        </tr>
                        <tr>
                            <th>Human Result</th>
                            <td>${result.human_result}</td>
                        </tr>
                    `)}
                `)}
            </table>
        `;
    }
}

customElements.define("dmarc-report", Report);
