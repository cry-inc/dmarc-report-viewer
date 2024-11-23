import { LitElement, html, css } from "lit";

export class MailTable extends LitElement {
    static styles = css`
        table {
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
        if (mail.xml_file_count < 1) {
            return html`<span class="problem">${mail.xml_file_count}</span>`;
        } else {
            return html`<span class="noproblem">${mail.xml_file_count}</span>`;;
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
                    <th>XMLs</th>
                </tr>
                ${this.mails.length !== 0 ? this.mails.map((mail) =>
                    html`<tr> 
                        <td><a href="#/mails/${mail.uid}">${this.prepareSubject(mail.subject)}</a></td>    
                        <td><a href="#/mails?sender=${encodeURIComponent(mail.sender)}">${mail.sender}</a></td>
                        <td>${new Date(mail.date * 1000).toLocaleString()}</td>
                        <td>${this.prepareSize(mail)}</td>
                        <td>${this.prepareXmlFileCount(mail)}</td>
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
