import toml
import json
import base64
import requests
import apple_cache

# Load and parse the TOML file
with open('mac.toml', 'r') as toml_file:
    mac_data = toml.load(toml_file)

# Convert to a JSON string
json_data = json.dumps(mac_data)

# Initialize the serial with the JSON data
apple_cache.init_serial(json_data)

# Load the JSON template from 'cache.json' file
with open('cache.json', 'r') as json_file:
    template = json.load(json_file)  # This now represents the JSON template

# Create a new CacheApi instance
cache = apple_cache.CacheApi()

# Call the create method and encode the response
res = cache.create(None)
encoded_res = base64.b64encode(bytes(res.data)).decode('utf-8')
print(f"+ res: {res.ctx}, {encoded_res}")

# Setup headers for the HTTP request
headers = {
    'User-Agent': 'AssetCache/243 CFNetwork/1111 Darwin/19.0.0 (x86_64)',
    'X-Protocol-Version': '3'
}

# Create a session to manage cookies and requests
session = requests.Session()
session.headers.update(headers)

# Disable SSL certificate verification (not recommended for production)
session.verify = False

# Send request to create a new session
resp = session.post("https://lcdn-registration.apple.com/lcdn/session", data=encoded_res)

# Check if the LCDN-Session cookie was successfully retrieved
lcdnsession_cookie = resp.cookies.get('LCDN-Session')
if not lcdnsession_cookie:
    raise Exception("LCDN-Session cookie not found.")

print(f"+ LCDN: {lcdnsession_cookie}")

# Obtain the response text, removing quotes and replacing escaped equals
data = resp.text.strip('"').replace("\\u003d", "=")

print(f"Got {data}")

# Decode the data from base64
decoded_data = base64.b64decode(data)

# Use the obtained data
cache.obtain(res.ctx, list(decoded_data))

# Use the LCDN-Session cookie to update the template
template['session-token'] = lcdnsession_cookie
data_to_register = json.dumps(template)

print(f"+ Send {data_to_register}")

# Sign the updated data
signed_data = cache.sign(res.ctx, list(data_to_register.encode('utf-8')))

# Encode the signed data
signed_data_encoded = base64.b64encode(bytes(signed_data)).decode('utf-8')
print(f"+ Sign: {signed_data_encoded}")

# Send the registration request
register_resp = session.post("https://lcdn-registration.apple.com/lcdn/register", data=signed_data_encoded)

# Check the registration response for errors
if register_resp.status_code != 200:
    raise Exception(f"Registration failed with status code {register_resp.status_code}: {register_resp.text}")

# Print the registration response
print(f"+ Register: {register_resp.text}")
