import { LitElement, html, css } from "lit";

export class MailTable extends LitElement {
    static styles = css`
        table {
            margin-top: 15px;
            width: 100%;
        }

        th {
            text-align: left;
            background-color: #efefef;
        }

        td, th {
            padding-left: 10px;
            padding-right: 10px;
            padding-top: 3px;
            padding-bottom: 3px;
        }

        tr:hover {
            background-color: #f4f4f4;
        }

        .problem {
            border-radius: 3px;
            padding-left: 4px;
            padding-right: 4px;
            color: white;
            background-color: #f00;
        }
            
        .noproblem {
            color: #ccc;
        }`;

    static properties = {
        mails: { type: Array },
    };

    constructor() {
        super();
        this.mails = [];
    }

    prepareSubject(subject) {
        const regex = /Report Domain: |Report domain: /;
        const prefix_removed = subject.replace(regex, "");
        if (prefix_removed.length < 90) {
            return prefix_removed;
        } else {
            return prefix_removed.substring(0, 75) + "...";
        }
    }

    prepareSize(mail) {
        if (mail.oversized) {
            return html`<span class="problem">${mail.size}</span>`;
        } else {
            return mail.size;
        }
    }

    prepareXmlFileCount(mail) {
        if (mail.oversized) {
            return html`<span class="noproblem">n/a</span>`;
        } else if (mail.xml_files < 1) {
            return html`<span class="problem">${mail.xml_files}</span>`;
        } else {
            return html`<span class="noproblem">${mail.xml_files}</span>`;
        }
    }

    prepareParsingErrorCount(mail) {
        if (mail.oversized) {
            return html`<span class="noproblem">n/a</span>`;
        } else if (mail.parsing_errors > 0) {
            return html`<span class="problem">${mail.parsing_errors}</span>`;
        } else {
            return html`<span class="noproblem">${mail.parsing_errors}</span>`;
        }
    }

    render() {
        return html`
            <table>
                <tr>
                    <th>Subject</th>
                    <th>Sender</th>
                    <th>Date</th>
                    <th>Size</th>
                    <th style="cursor:help" title="XML File Count">XMLs</th>
                    <th style="cursor:help" title="Number of XML DMARC Parsing Errors">Errors</th>
                </tr>
                ${this.mails.length !== 0 ? this.mails.map((mail) =>
                    html`<tr> 
                        <td><a href="#/mails/${mail.uid}">${this.prepareSubject(mail.subject)}</a></td>    
                        <td><a href="#/mails?sender=${encodeURIComponent(mail.sender)}">${mail.sender}</a></td>
                        <td>${new Date(mail.date * 1000).toLocaleString()}</td>
                        <td>${this.prepareSize(mail)}</td>
                        <td>${this.prepareXmlFileCount(mail)}</td>
                        <td>${this.prepareParsingErrorCount(mail)}</td>
                    </tr>`
                ) : html`<tr>
                        <td colspan="4">No mails found.</td>
                    </tr>`
                }
            </table>
        `;
    }
}

customElements.define("dmarc-mail-table", MailTable);
