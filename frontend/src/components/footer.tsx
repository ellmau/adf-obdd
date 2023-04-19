import React, {
  useState, useCallback, useContext, useEffect, useRef,
} from 'react';

import {
  AlertColor,
  Alert,
  AppBar,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Link,
  TextField,
  Toolbar,
} from '@mui/material';

import LoadingContext from './loading-context';
import SnackbarContext from './snackbar-context';

enum UserFormType {
  Login = 'Login',
  Register = 'Register',
  Update = 'Update',
}

interface UserFormProps {
  formType: UserFormType | null;
  close: (message?: string, severity?: AlertColor) => void;
  username?: string;
}

function UserForm({ username: propUsername, formType, close }: UserFormProps) {
  const { setLoading } = useContext(LoadingContext);
  const [username, setUsername] = useState<string>(propUsername || '');
  const [password, setPassword] = useState<string>('');
  const [errorOccurred, setError] = useState<boolean>(false);

  const submitHandler = useCallback(
    (del: boolean) => {
      setLoading(true);
      setError(false);

      let method; let
        endpoint;
      if (del) {
        method = 'DELETE';
        endpoint = '/users/delete';
      } else {
        switch (formType) {
          case UserFormType.Login:
            method = 'POST';
            endpoint = '/users/login';
            break;
          case UserFormType.Register:
            method = 'POST';
            endpoint = '/users/register';
            break;
          case UserFormType.Update:
            method = 'PUT';
            endpoint = '/users/update';
            break;
          default:
            // NOTE: the value is not null when the dialog is open
            break;
        }
      }

      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}${endpoint}`, {
        method,
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        headers: {
          'Content-Type': 'application/json',
        },
        body: !del ? JSON.stringify({ username, password }) : undefined,
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              close(`Action '${del ? 'Delete' : formType}' successful!`, 'success');
              break;
            default:
              setError(true);
              break;
          }
        })
        .finally(() => setLoading(false));
    },
    [username, password, formType],
  );

  return (
    <form onSubmit={(e) => { e.preventDefault(); submitHandler(false); }}>
      <DialogTitle>{formType}</DialogTitle>
      <DialogContent>
        <TextField
          variant="standard"
          type="text"
          label="Username"
          value={username}
          onChange={(event) => { setUsername(event.target.value); }}
        />
        <br />
        <TextField
          variant="standard"
          type="password"
          label="Password"
          value={password}
          onChange={(event) => { setPassword(event.target.value); }}
        />
        {errorOccurred
          && <Alert severity="error">Check your inputs!</Alert>}
      </DialogContent>
      <DialogActions>
        <Button type="button" onClick={() => close()}>Cancel</Button>
        <Button type="submit" variant="contained" color="success">{formType}</Button>
        {formType === UserFormType.Update
        // TODO: add another confirm dialog here
          && (
          <Button
            type="button"
            variant="outlined"
            color="error"
            onClick={() => {
              // eslint-disable-next-line no-alert
              if (window.confirm('Are you sure that you want to delete your account?')) {
                submitHandler(true);
              }
            }}
          >
            Delete Account
          </Button>
          )}
      </DialogActions>
    </form>
  );
}

UserForm.defaultProps = { username: undefined };

function Footer() {
  const { status: snackbarInfo, setStatus: setSnackbarInfo } = useContext(SnackbarContext);
  const [username, setUsername] = useState<string>();
  const [tempUser, setTempUser] = useState<boolean>();
  const [dialogTypeOpen, setDialogTypeOpen] = useState<UserFormType | null>(null);

  const isFirstRender = useRef(true);

  const logout = useCallback(() => {
    fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/users/logout`, {
      method: 'DELETE',
      credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
      headers: {
        'Content-Type': 'application/json',
      },
    })
      .then((res) => {
        switch (res.status) {
          case 200:
            setSnackbarInfo({ message: 'Logout successful!', severity: 'success', potentialUserChange: true });
            setUsername(undefined);
            break;
          default:
            setSnackbarInfo({ message: 'An error occurred while trying to log out.', severity: 'error', potentialUserChange: false });
            break;
        }
      });
  }, [setSnackbarInfo]);

  useEffect(() => {
    // TODO: having the info if the user may have changed on the snackbar info
    // is a bit lazy and unclean; be better!
    if (isFirstRender.current || snackbarInfo?.potentialUserChange) {
      isFirstRender.current = false;

      fetch(`${process.env.NODE_ENV === 'development' ? '//localhost:8080' : ''}/users/info`, {
        method: 'GET',
        credentials: process.env.NODE_ENV === 'development' ? 'include' : 'same-origin',
        headers: {
          'Content-Type': 'application/json',
        },
      })
        .then((res) => {
          switch (res.status) {
            case 200:
              res.json().then(({ username: user, temp }) => {
                setUsername(user);
                setTempUser(temp);
              });
              break;
            default:
              setUsername(undefined);
              break;
          }
        });
    }
  }, [snackbarInfo?.potentialUserChange]);

  return (
    <>
      <AppBar position="fixed" sx={{ top: 'auto', bottom: 0 }}>
        <Toolbar sx={{ justifyContent: 'center', alignItems: 'center' }}>
          <Box sx={{ flexGrow: 1 }}>
            {username ? (
              <>
                <span>
                  Logged in as:
                  {' '}
                  {username}
                  {' '}
                  {tempUser ? '(Temporary User. Edit to set a password!)' : undefined}
                </span>
                <Button color="inherit" onClick={() => setDialogTypeOpen(UserFormType.Update)}>Edit</Button>
                {!tempUser && <Button color="inherit" onClick={() => logout()}>Logout</Button>}
              </>
            ) : (
              <>
                <Button color="inherit" onClick={() => setDialogTypeOpen(UserFormType.Login)}>Login</Button>
                <Button color="inherit" onClick={() => setDialogTypeOpen(UserFormType.Register)}>Register</Button>
              </>
            )}
          </Box>

          <Link href="/legal.html" target="_blank" sx={{ fontSize: '0.8rem' }}>
            Legal Information (Impressum and Data Protection Regulation)
          </Link>
        </Toolbar>
      </AppBar>
      <Dialog open={!!dialogTypeOpen} onClose={() => setDialogTypeOpen(null)}>
        <UserForm
          formType={dialogTypeOpen}
          close={(message, severity) => {
            setDialogTypeOpen(null);
            setSnackbarInfo((!!message && !!severity)
              ? { message, severity, potentialUserChange: true }
              : undefined);
          }}
          username={dialogTypeOpen === UserFormType.Update ? username : undefined}
        />
      </Dialog>
    </>
  );
}

export default Footer;
