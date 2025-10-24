package plugin

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"time"
)

// Register registers the plugin with the plugin host and returns an authentication token. Upon successful registration of the plugin, the Authorisation header is set on every protected route to the backend, which already contains this token.
//
// This function performs the following steps:
//  1. Constructs a registration request payload containing plugin metadata (such as slug, name, version, and description)
//     and authentication credentials (client ID and client secret).
//  2. Sends the registration request as an HTTP POST request to the plugin host's registration API endpoint.
//  3. Parses the response from the plugin host to extract and return the authentication tokens.
//
// Parameters:
// - ctx: The context for managing request deadlines and cancellation.
//
// Returns:
//   - A pointer to a Token struct containing the access and refresh tokens if registration is successful.
//   - An error if the registration fails, due to either an HTTP request issue, an unexpected status code,
//     or failure in parsing the response body.
//
// Possible errors:
// - If the request payload cannot be marshaled into JSON, an error is returned.
// - If creating the HTTP request fails, an error is returned.
// - If the HTTP request fails (e.g., network error), an error is returned.
// - If the plugin host returns a non-200 status code, an error is returned with a generic failure message.
// - If the response body cannot be decoded into the expected format, an error is returned.
//
// Example usage:
//
//	token, err := pluginWorker.Register(ctx)
//	if err != nil {
//		log.Fatalf("Failed to register plugin: %v", err)
//	}
//	log.Printf("Successfully registered. Access token: %s", token.AccessToken)
func (w *PluginWorker) Register(ctx context.Context) (*Token, error) {
	reqBody := PluginRegisterRequest{
		Slug:        w.cfg.plugin.Slug,
		Name:        w.cfg.plugin.Name,
		Path:        w.cfg.plugin.PluginHostPath.String(),
		Version:     w.cfg.plugin.Version,
		Description: w.cfg.plugin.Description,
		Auth: PluginAuth{
			ClientID:     w.cfg.clientID,
			ClientSecret: w.cfg.clientSecret,
		},
	}

	buf, err := json.Marshal(reqBody)
	if err != nil {
		return nil, err
	}

	registerPath := fmt.Sprintf("%s/api/%s/plugin/register", w.cfg.host, w.cfg.hostAPIVersion)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, registerPath, bytes.NewReader(buf))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	resp, err := w.cfg.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, errors.New("failed to register plugin")
	}

	var tokenResp ClientTokenResponse
	if err := json.NewDecoder(resp.Body).Decode(&tokenResp); err != nil {
		return nil, err
	}

	token := &Token{
		AccessToken:  tokenResp.AccessToken,
		RefreshToken: tokenResp.RefreshToken,
		ExpiresIn:    int64(tokenResp.ExpiresIn),
		Expiry:       tokenResp.Expiry,
		TokenType:    tokenResp.TokenType,
	}

	w.cfg.token = token
	return token, nil
}

// Unregister removes the plugin registration from the plugin host. The authorisation header will be set to internal hold access token.
//
// This function performs the following steps:
//  1. Constructs the API endpoint URL for unregistering the plugin based on the plugin's configuration.
//  2. Creates an HTTP POST request with the given context to initiate the unregistration process.
//  3. Sends the request using the configured HTTP client and handles any potential errors.
//  4. Validates the response status code to ensure successful unregistration.
//
// Parameters:
// - ctx: The context for managing request deadlines and cancellation.
//
// Returns:
// - nil if the unregistration is successful.
// - An error if the request fails, the HTTP client encounters an issue, or the response status code is not 204 No Content.
//
// Possible errors:
// - If creating the HTTP request fails, an error is returned.
// - If the HTTP request fails (e.g., network error), an error is returned.
// - If the plugin host returns a non-204 status code, an error is returned with a generic failure message.
//
// Example usage:
//
//	err := pluginWorker.Unregister(ctx)
//	if err != nil {
//	    log.Fatalf("Failed to unregister plugin: %v", err)
//	}
//	log.Println("Successfully unregistered plugin")
func (w *PluginWorker) Unregister(ctx context.Context) error {
	unregisterPath := fmt.Sprintf("%s://%s/api/%s/plugin/%s/unregister", w.cfg.host.Scheme, w.cfg.host.Host, w.cfg.hostAPIVersion, w.cfg.plugin.Slug)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, unregisterPath, nil)
	if err != nil {
		return err
	}

	w.checkAndRenewToken(ctx)
	req.Header.Set("Authorization", fmt.Sprintf("%s %s", w.cfg.token.TokenType, w.cfg.token.AccessToken))
	resp, err := w.cfg.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusNoContent {
		return errors.New("failed to unregister plugin")
	}

	return nil
}

// RefreshToken refreshes the authentication token for the plugin.
func (w *PluginWorker) RefreshToken(ctx context.Context) (*Token, error) {
	reqBody := PluginAuth{
		ClientID:     w.cfg.clientID,
		ClientSecret: w.cfg.clientSecret,
	}

	buf, err := json.Marshal(reqBody)
	if err != nil {
		return nil, err
	}

	refreshPath := fmt.Sprintf("%s/api/%s/plugin/%s/token/refresh", w.cfg.host, w.cfg.hostAPIVersion, w.cfg.plugin.Slug)
	fmt.Println(refreshPath)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, refreshPath, bytes.NewReader(buf))
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	resp, err := w.cfg.client.Do(req)
	fmt.Println(resp, err)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, errors.New("failed to refresh plugin token")
	}

	var tokenResp ClientTokenResponse
	if err := json.NewDecoder(resp.Body).Decode(&tokenResp); err != nil {
		return nil, err
	}

	token := &Token{
		AccessToken:  tokenResp.AccessToken,
		RefreshToken: tokenResp.RefreshToken,
		ExpiresIn:    int64(tokenResp.ExpiresIn),
		Expiry:       tokenResp.Expiry,
		TokenType:    tokenResp.TokenType,
	}

	w.cfg.token = token
	return token, nil
}

// Heartbeat sends a periodic heartbeat signal to the plugin host to indicate that the plugin is active and operational. The authorisation header will be set to internal hold access token.
//
// This function performs the following steps:
// 1. Constructs the heartbeat API endpoint URL based on the plugin's slug and configuration settings.
// 2. Sends an HTTP POST request to the constructed endpoint without a request body.
// 3. Checks the response status code to confirm the success of the heartbeat operation.
//
// Parameters:
// - ctx: The context for managing request deadlines and cancellation.
//
// Returns:
// - nil if the heartbeat is successfully sent and acknowledged by the plugin host.
// - An error if any of the following issues occur:
//   - Constructing the HTTP request fails.
//   - Sending the HTTP request fails (e.g., due to network issues).
//   - The plugin host responds with a non-200 status code, indicating the heartbeat was not successfully received.
//
// Example usage:
//
//	err := pluginWorker.Heartbeat(ctx)
//	if err != nil {
//		log.Printf("Failed to send heartbeat: %v", err)
//	} else {
//		log.Println("Heartbeat sent successfully.")
//	}
//
// Notes:
// - The plugin's slug, host, and API version must be correctly configured in the `PluginWorker` instance.
// - This function assumes that the plugin host's heartbeat API endpoint requires an HTTP POST request.
func (w *PluginWorker) Heartbeat(ctx context.Context) error {
	heartbeatPath := fmt.Sprintf("%s/api/%s/plugin/%s/heartbeat", w.cfg.host, w.cfg.hostAPIVersion, w.cfg.plugin.Slug)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, heartbeatPath, http.NoBody)
	if err != nil {
		return err
	}

	w.checkAndRenewToken(ctx)
	req.Header.Set("Authorization", fmt.Sprintf("%s %s", w.cfg.token.TokenType, w.cfg.token.AccessToken))
	resp, err := w.cfg.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return errors.New("failed to send heartbeat")
	}

	return nil
}

func (w *PluginWorker) checkAndRenewToken(ctx context.Context) error {
	if w.cfg.token.Expiry.Before(time.Now()) {
		newToken, err := w.RefreshToken(ctx)
		if err != nil {
			return err
		}

		w.cfg.token = newToken
	}

	return nil
}
