import { css } from "lit";

export const globalStyle = css`
    a {
        color: rgb(0, 123, 255);
        text-decoration: none;
    }

    a:hover {
        color: rgb(0, 86, 179);
    }

    .badge {
        border-radius: 3px;
        padding-left: 4px;
        padding-right: 4px;
        background-color: #888;
        color: white;
    }

    .badge-negative {
        background-color: rgb(220, 53, 69);
    }

    .badge-positive {
        background-color: rgb(25, 135, 84);
    }

    .noproblem {
        color: #ccc;
    }

    .help {
        cursor: help;
    }

    table {
        width: 100%;
        margin-top: 15px;
    }

    th {
        text-align: left;
        background-color: #efefef;
    }

    td, th {
        padding-left: 15px;
        padding-right: 15px;
        padding-top: 3px;
        padding-bottom: 3px;
    }

    table:not(.vertical) tr:hover {
        background-color: #f4f4f4;
    }

    table.vertical th {
        width: 175px;
    }

    h1, h2, h3 {
        padding: 0px;
        margin-top: 15px;
        margin-bottom: 15px;
    }

    h1 {
        margin-top: 0px;
    }

    .button {
        display: inline-block;
        padding: 5px;
        padding-left: 8px;
        padding-right: 8px;
        margin-right: 10px;
        color: white;
        border-radius: 3px;
        background-color: rgb(108, 117, 125);
    }

    .button:hover {
        color: white;
        background-color: rgb(90, 98, 104);
    }

    .ml {
        margin-left: 10px;
    }
`;
