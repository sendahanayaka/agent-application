import { Blocks, ArrowUpRight } from 'lucide-react';
import { ToolInfo } from '../types/api';

const examples: Record<string,string> = {
  get_all_employees: 'Show all employees', get_employee_by_id: 'Find employee ID 1003',
  search_employees: 'Search employees in IT department', get_department_list: 'Show all departments',
  get_employee_count: 'How many employees are there?', add_employee: 'Add a new employee named Alice Smith',
  update_employee: 'Update employee 1001 department to IT',
};
interface Props { tools: ToolInfo[]; isLoading: boolean; onSelectPrompt: (prompt: string) => void; }
export function ToolsPanel({tools, isLoading, onSelectPrompt}: Props) {
  return <main className="library"><h1>Tool library</h1>
    <p>{isLoading ? 'Loading tools…' : tools.length ? `${tools.length} available tools. Inspect their inputs or start a chat with an example.` : 'No tools are available. Check the connection status above.'}</p>
    <div className="tool-grid">{tools.map(tool => <article key={tool.name} className="tool-card">
      <header><Blocks size={19}/><span>{(tool.annotations?.readOnlyHint ?? !['add_employee','update_employee'].includes(tool.name)) ? 'Read' : 'Requires confirmation'}</span></header>
      <h2>{tool.title || tool.name.replace(/_/g,' ')}</h2><p>{tool.description}</p>
      <details><summary>Input schema</summary><pre>{JSON.stringify(tool.inputSchema, null, 2)}</pre></details>
      {examples[tool.name] && <button className="text-action" onClick={() => onSelectPrompt(examples[tool.name])}>Try in chat <ArrowUpRight size={15}/></button>}
    </article>)}</div>
  </main>;
}
