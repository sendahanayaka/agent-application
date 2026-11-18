import React from 'react';
import { PendingOperation } from '../types/api';

interface ConfirmationCardProps {
  operation: PendingOperation;
  isConfirming: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

const FIELD_LABELS: Record<string, string> = {
  employee_id: 'Target Employee ID',
  first_name: 'First Name',
  last_name: 'Last Name',
  email: 'Corporate Email',
  phone: 'Phone Number',
  department_id: 'Department ID',
  designation_id: 'Designation ID',
  salary: 'Annual Salary',
  hire_date: 'Hire Date',
  status: 'Employment Status',
};

export const ConfirmationCard: React.FC<ConfirmationCardProps> = ({
  operation,
  isConfirming,
  onConfirm,
  onCancel,
}) => {
  const isUpdate = operation.tool_name === 'update_employee';
  const args = operation.arguments || {};



  const formatValue = (key: string, val: any): string => {
    if (val === null || val === undefined) return 'None';
    if (key === 'salary') return `$${Number(val).toLocaleString()}`;
    if (key === 'phone') return String(val); // Preserve leading zeros!
    return String(val);
  };

  return (
    <section className="review">
      <h4>{isUpdate ? 'Review employee update' : 'Review new employee'}</h4>
      <p>Confirm to save these details.</p>
      <dl>{Object.entries(args).map(([key, value]) => (
        <React.Fragment key={key}><dt>{FIELD_LABELS[key] || key}</dt><dd>{formatValue(key, value)}</dd></React.Fragment>
      ))}</dl>
      <footer><button className="approve" onClick={onConfirm} disabled={isConfirming}>{isConfirming ? 'Saving…' : 'Confirm changes'}</button>
        <button onClick={onCancel} disabled={isConfirming}>Cancel</button></footer>
    </section>
  );
};
