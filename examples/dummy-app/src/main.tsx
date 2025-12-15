import ReactDOM from "react-dom/client";
import { AuthProvider, useAuth } from "react-oidc-context";

/**
 * ============================
 * Realm / Environment Config
 * ============================
 */
const REALM = "customers";
const REAUTH_BASE_URL = "http://localhost:3000";
const REALM_BASE_URL = `${REAUTH_BASE_URL}/api/realms/${REALM}`;

/**
 * ============================
 * OIDC Configuration
 * ============================
 */
const oidcConfig = {
  authority: REALM_BASE_URL,

  client_id: "dummy-app-2",
  redirect_uri: "http://localhost:6565",

  // Explicit metadata (no .well-known yet)
  metadata: {
    issuer: REALM_BASE_URL,
    authorization_endpoint: `${REALM_BASE_URL}/oidc/authorize`,
    token_endpoint: `${REALM_BASE_URL}/oidc/token`,
    userinfo_endpoint: `${REALM_BASE_URL}/oidc/userinfo`,
  },

  onSigninCallback: () => {
    window.history.replaceState({}, document.title, window.location.pathname);
  },
};

function App() {
  const auth = useAuth();

  if (auth.isLoading) {
    return <div>Loading Auth...</div>;
  }

  if (auth.error) {
    return <div>Oops... {auth.error.message}</div>;
  }

  if (auth.isAuthenticated) {
    return (
      <div style={{ padding: 20 }}>
        <h1>Hello {auth.user?.profile.preferred_username || "User"}!</h1>
        <p>You are logged in.</p>

        <button onClick={() => auth.removeUser()}>Log out</button>

        <h3>Your Access Token:</h3>
        <pre style={{ background: "#f4f4f4", padding: 10 }}>
          {auth.user?.access_token}
        </pre>
      </div>
    );
  }

  return (
    <div style={{ padding: 20 }}>
      <h1>Welcome to the Dummy App</h1>
      <button onClick={() => auth.signinRedirect()}>Log in with ReAuth</button>
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <AuthProvider {...oidcConfig}>
    <App />
  </AuthProvider>,
);
