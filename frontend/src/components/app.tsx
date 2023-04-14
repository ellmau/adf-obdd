import React, { useState, useMemo } from 'react';

import { createBrowserRouter, RouterProvider } from 'react-router-dom';

import { ThemeProvider, createTheme } from '@mui/material/styles';
import {
  Alert,
  AlertColor,
  Backdrop,
  CircularProgress,
  CssBaseline,
  Snackbar,
  useMediaQuery,
} from '@mui/material';

import LoadingContext from './loading-context';
import SnackbarContext from './snackbar-context';
import Footer from './footer';
import AdfOverview from './adf-overview';
import AdfDetails from './adf-details';

const browserRouter = createBrowserRouter([
  {
    path: '/',
    element: <AdfOverview />,
  },
  {
    path: '/:adfName',
    element: <AdfDetails />,
  },
]);

function App() {
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');

  const theme = useMemo(
    () => createTheme({
      palette: {
        mode: prefersDarkMode ? 'dark' : 'light',
      },
    }),
    [prefersDarkMode],
  );

  const [loading, setLoading] = useState(false);
  const loadingContext = useMemo(() => ({ loading, setLoading }), [loading, setLoading]);

  const [snackbarInfo, setSnackbarInfo] = useState<{
    message: string,
    severity: AlertColor,
    potentialUserChange: boolean,
  } | undefined>();
  const snackbarContext = useMemo(
    () => ({ status: snackbarInfo, setStatus: setSnackbarInfo }),
    [snackbarInfo, setSnackbarInfo],
  );

  return (
    <ThemeProvider theme={theme}>
      <LoadingContext.Provider value={loadingContext}>
        <SnackbarContext.Provider value={snackbarContext}>
          <CssBaseline />
          <main style={{ maxHeight: 'calc(100vh - 70px)', overflowY: 'auto' }}>
            <RouterProvider router={browserRouter} />
          </main>
          <Footer />

          <Backdrop
            open={loading}
          >
            <CircularProgress color="inherit" />
          </Backdrop>
          <Snackbar
            open={!!snackbarInfo}
            autoHideDuration={10000}
            onClose={() => setSnackbarInfo(undefined)}
          >
            <Alert severity={snackbarInfo?.severity}>{snackbarInfo?.message}</Alert>
          </Snackbar>
        </SnackbarContext.Provider>
      </LoadingContext.Provider>
    </ThemeProvider>
  );
}

export default App;
