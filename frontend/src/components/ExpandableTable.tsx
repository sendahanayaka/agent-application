import { useId, useRef, useState, type ReactNode } from 'react';
import { Maximize2, X } from 'lucide-react';

interface Props {
  title: string;
  heading: ReactNode;
  children: ReactNode;
}

export function ExpandableTable({ title, heading, children }: Props) {
  const titleId = useId();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <section className="result-table">
      <div className="result-table-bar">
        {heading}
        <button
          type="button"
          className="table-action"
          aria-label={`Expand ${title}`}
          aria-haspopup="dialog"
          onClick={() => {
            dialogRef.current?.showModal();
            setIsExpanded(true);
          }}
        >
          <Maximize2 size={14} aria-hidden="true" /> Expand
        </button>
      </div>
      <div className="table-scroll table-scroll-inline" tabIndex={0} role="region" aria-label={title}>
        {children}
      </div>
      <dialog
        ref={dialogRef}
        className="expanded-table"
        aria-labelledby={titleId}
        onClose={() => setIsExpanded(false)}
      >
        <div className="result-table-bar">
          <h3 id={titleId}>{title}</h3>
          <button type="button" className="table-action" onClick={() => dialogRef.current?.close()} autoFocus>
            <X size={16} aria-hidden="true" /> Close
          </button>
        </div>
        {isExpanded && (
          <div className="table-scroll" tabIndex={0} role="region" aria-label={title}>
            {children}
          </div>
        )}
      </dialog>
    </section>
  );
}
