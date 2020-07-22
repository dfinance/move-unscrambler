{{~#*inline "address"~}}
{{#if this.0.1}}{{this.0.0}}::{{this.0.1}}{{/if}}{{#if this.1}}::{{this.1}}{{/if}}
{{/inline}}



{{#*inline "function" no_title=false}}
{{#if no_title}}{{else}}## Function {{> address this.address}} {{/if}}
{{#if acquires.0}}
Acquires: {{#each acquires}} {{>address}} {{/each}}
{{/if}}
{{#if calls.0}}
Calls: {{#each calls}} {{>address}} {{/each}}
{{/if}}
{{/inline}}



{{#*inline "struct"}}
## Struct: {{this.address}}
...
{{/inline}}



<!-- root: -->

{{#if root.is_script }}
# Transaction script

{{>function root.entry_points.0 no_title=true}}



{{else}}
# Module {{ root.address }}

{{#each root.entry_points}}
	{{log @index}}
	{{~log this ~}}
	{{> function}}
{{/each}}

{{/if}}



<!-- dependencies: -->

{{#if dependencies.functions}}
# Dependencies: functions

{{#each dependencies.functions}}
{{> function}}
{{/each}}
{{/if}}



{{#if dependencies.structs}}
# Dependencies: structs

{{#each dependencies.structs}}
{{> struct}}
{{/each}}
{{/if}}
