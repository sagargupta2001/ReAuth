import ReactDOM from "react-dom/client";
import { AuthProvider, useAuth } from "react-oidc-context";

const oidcConfig = {
  // 1. Point to ReAuth
  authority: "http://localhost:3000/api/realms/master",

  // 2. Client Details
  client_id: "dummy-app",
  redirect_uri: "http://localhost:6565",

  // 3. IMPORTANT: ReAuth might not have a discovery endpoint yet (.well-known)
  // If not, we must manually define the metadata:
  metadata: {
    issuer: "http://localhost:3000/api/realms/master",
    authorization_endpoint:
      "http://localhost:3000/api/realms/master/oidc/authorize",
    token_endpoint: "http://localhost:3000/api/realms/master/oidc/token",
    userinfo_endpoint: "http://localhost:3000/api/realms/master/oidc/userinfo",
  },

  // 4. PKCE is mandatory for SPAs in 2025
  onSigninCallback: () => {
    // Remove query params like ?code=... after successful login
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
  </AuthProvider>
);
