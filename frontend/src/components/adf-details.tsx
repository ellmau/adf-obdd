import React from 'react';

import { AlertColor } from '@mui/material';

import { GraphProps } from './graph-g6';

export type Parsing = 'Naive' | 'Hybrid';

export type StrategySnakeCase = 'parse_only' | 'ground' | 'complete' | 'stable' | 'stable_counting_a' | 'stable_counting_b' | 'stable_nogood';

export type StrategyCamelCase = 'ParseOnly' | 'Ground' | 'Complete' | 'Stable' | 'StableCountingA' | 'StableCountingB' | 'StableNogood';

export interface AcAndGraph {
  ac: string[],
  graph: GraphProps,
}

export type AcsWithGraphsOpt = {
  type: 'None',
} | {
  type: 'Error',
  content: string
} | {
  type: 'Some',
  content: AcAndGraph[]
};

export type Task = {
  type: 'Parse',
} | {
  type: 'Solve',
  content: StrategyCamelCase,
};

export interface AdfProblemInfo {
  name: string,
  code: string,
  parsing_used: Parsing,
  // NOTE: the keys are really only strategies
  acs_per_strategy: { [key in StrategySnakeCase]: AcsWithGraphsOpt },
  running_tasks: Task[],
}

export function acsWithGraphOptToColor(status: AcsWithGraphsOpt, running: boolean): AlertColor {
  if (running) {
    return 'warning';
  }

  switch (status.type) {
    case 'None': return 'info';
    case 'Error': return 'error';
    case 'Some': return 'success';
    default:
      throw new Error('Unknown type union variant (cannot occur)');
  }
}

export function acsWithGraphOptToText(status: AcsWithGraphsOpt, running: boolean): string {
  if (running) {
    return 'Running';
  }

  switch (status.type) {
    case 'None': return 'Not attempted';
    case 'Error': return 'Failed';
    case 'Some': return 'Done';
    default:
      throw new Error('Unknown type union variant (cannot occur)');
  }
}

function AdfDetails() {
  return (
    <div>Details</div>
  );
}

export default AdfDetails;
