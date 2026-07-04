const fs = require("fs");
const path = require("path");
const vm = require("vm");

const root = path.resolve(__dirname, "..");
const template = fs.readFileSync(path.join(root, "template.html"), "utf8");
const match = template.match(/<script>\s*([\s\S]*?)\s*<\/script>\s*$/);

if (!match) {
  throw new Error("template.html script block not found");
}

new vm.Script(match[1], { filename: "template.html<script>" });
console.log("PASS: template script syntax");
