import { createContext } from 'react';

import { AlertColor } from '@mui/material';

type Status = { message: string, severity: AlertColor, potentialUserChange: boolean } | undefined;

interface ISnackbarContext {
  status: Status;
  setStatus: (status: Status) => void;
}

const SnackbarContext = createContext<ISnackbarContext>({
  status: undefined,
  setStatus: () => {},
});

export default SnackbarContext;
