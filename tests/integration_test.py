#!/usr/bin/env python3

import os
import sys
import time
import random
import signal
import string
import subprocess
import requests
import sqlite3
import tempfile
import json

def generate_secret_key(length=32):
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

def wait_for_server(host, port, timeout=5):
    start_time = time.time()
    while True:
        try:
            response = requests.get(f'http://{host}:{port}/health_check')
            if response.status_code == 200:
                return
        except requests.exceptions.ConnectionError:
            pass

        if time.time() - start_time > timeout:
            raise AssertionError("Not connected")

        time.sleep(0.1)

def test_create_session_without_body(BASE_URL, cursor):
    try:
        response = requests.post(f'{BASE_URL}/create_session', json={})
        assert response.status_code == 200, f"Expected status code 200, got {response.status_code}"

        response_data = response.json()
        if 'session_id' not in response_data:
            print('⍜ Test 1 Failed: session_id not in response')
            return None
        else:
            session_id = response_data['session_id']
            user_id = response_data['user_id']

            print(f'⦿ Test 1 Passed: session_id = {session_id}, user_id = {user_id}')

            # Check database
            cursor.execute("SELECT * FROM sessions WHERE session_id = ?", (session_id,))
            row = cursor.fetchone()
            if row is None:
                print('⍜ Test 1 Failed: session_id not found in database')
            else:
                ip_address = row['ip_address']
                if ip_address != '127.0.0.1':
                    print(f'⍜ Test 1 Failed: ip_address is {ip_address}, expected 127.0.0.1')
                else:
                    print('⦿ Test 1 Passed: session_id found in database with correct ip_address')

                user_id_test = row['user_id']
                if user_id_test != user_id:
                    print(f'⍜ Test 1 Failed: user_id is {user_id_test}, expected 127.0.0.1')
                else:
                    print('⦿ Test 1 Passed: session_id found in database with correct user_id')                    
            return session_id
    except Exception as e:
        print(f'⍜ Test 1 Failed: {e}')
        return None

def test_create_session_with_user_id(BASE_URL, cursor):
    try:
        response = requests.post(f'{BASE_URL}/create_session', json={ "user_id": 'xxx' })
        assert response.status_code == 200, f"Expected status code 200, got {response.status_code}"

        response_data = response.json()
        if 'session_id' not in response_data:
            print('⍜ Test 8 Failed: session_id not in response')
            return None
        else:
            session_id = response_data['session_id']
            user_id = response_data['user_id']
            print(f'⦿ Test 8 Passed: session_id = {session_id}, user_id = {user_id}')
            # Check database
            cursor.execute("SELECT * FROM sessions WHERE session_id = ?", (session_id,))
            row = cursor.fetchone()
            if row is None:
                print('⍜ Test 8 Failed: session_id not found in database')
            else:
                ip_address = row['ip_address']
                if ip_address != '127.0.0.1':
                    print(f'⍜ Test 8 Failed: ip_address is {ip_address}, expected 127.0.0.1')
                else:
                    print('⦿ Test 8 Passed: session_id found in database with correct ip_address')

                user_id_test = row['user_id']
                if user_id_test != 'xxx':
                    print(f'⍜ Test 8 Failed: user_id is {user_id_test}, expected xxx')
                else:
                    print('⦿ Test 8 Passed: session_id found in database with correct user_id')

                
            return session_id
    except Exception as e:
        print(f'⍜ Test 8 Failed: {e}')
        return None


def test_create_session_with_body(BASE_URL, cursor):
    session_data = {
        'device_model': 'TestModel',
        'operating_system': 'TestOS',
        'screen_width': 1920,
        'screen_height': 1080
    }
    try:
        response = requests.post(f'{BASE_URL}/create_session', json=session_data)
        response_data = response.json()
        if 'session_id' not in response_data:
            print('⍜ Test 2 Failed: session_id not in response')
            return None
        else:
            session_id = response_data['session_id']
            print(f'⦿ Test 2 Passed: session_id = {session_id}')
            # Check database
            cursor.execute("SELECT * FROM sessions WHERE session_id = ?", (session_id,))
            row = cursor.fetchone()
            if row is None:
                print('⍜ Test 2 Failed: session_id not found in database')
            else:
                ip_address = row['ip_address']
                if ip_address != '127.0.0.1':
                    print(f'⍜ Test 2 Failed: ip_address is {ip_address}, expected 127.0.0.1')
                else:
                    # Check the other fields
                    device_model = row['device_model']
                    operating_system = row['operating_system']
                    screen_width = row['screen_width']
                    screen_height = row['screen_height']
                    if (device_model == 'TestModel' and
                        operating_system == 'TestOS' and
                        screen_width == 1920 and
                        screen_height == 1080):
                        print('⦿ Test 2 Passed: session data stored correctly in database')
                    else:
                        print('⍜ Test 2 Failed: session data not stored correctly in database')
            return session_id
    except Exception as e:
        print(f'⍜ Test 2 Failed: {e}')
        return None

def test_create_session_with_large_body(BASE_URL, MAX_JSON_PAYLOAD):
    large_data = 'x' * (int(MAX_JSON_PAYLOAD) + 1)
    try:
        response = requests.post(f'{BASE_URL}/create_session', json={'data': large_data})
        if response.status_code == 413:  # Payload Too Large
            print('⦿ Test 3 Passed: Request failed with status code 413 Payload Too Large')
        else:
            print(f'⍜ Test 3 Failed: Expected status code 413, got {response.status_code}')
    except Exception as e:
        print(f'⍜ Test 3 Failed: {e}')

def test_ingest_event_invalid_session_id(BASE_URL):
    invalid_session_id = 'invalid_session_id'
    event_data = {
        'session_id': invalid_session_id,
        'event_name': 'test-event'
    }
    try:
        response = requests.post(f'{BASE_URL}/ingest_event', json=event_data)
        if response.status_code != 200:
            print('⦿ Test 4 Passed: Request failed as expected')
        else:
            print('⍜ Test 4 Failed: Request succeeded unexpectedly')
    except Exception as e:
        print(f'⍜ Test 4 Failed: {e}')

def test_ingest_event_missing_event_name(BASE_URL, session_id):
    event_data = {
        'session_id': session_id
    }
    try:
        response = requests.post(f'{BASE_URL}/ingest_event', json=event_data)
        if response.status_code != 200:
            print('⦿ Test 5 Passed: Request failed as expected due to missing event_name')
        else:
            print('⍜ Test 5 Failed: Request succeeded unexpectedly')
    except Exception as e:
        print(f'⍜ Test 5 Failed: {e}')

def test_ingest_event_with_event_name(BASE_URL, cursor, session_id):
    event_data = {
        'session_id': session_id,
        'event_name': 'test-event'
    }
    try:
        response = requests.post(f'{BASE_URL}/ingest_event', json=event_data)
        if response.status_code == 200:
            print('⦿ Test 6 Passed: Event ingested successfully')
            # Check the database
            cursor.execute("SELECT * FROM events WHERE session_id = ? AND event_name = ?", (session_id, 'test-event'))
            row = cursor.fetchone()
            if row is None:
                print('⍜ Test 6 Failed: Event not found in database')
            else:
                ip_address = row['ip_address']
                if ip_address != '127.0.0.1':
                    print(f'⍜ Test 6 Failed: ip_address is {ip_address}, expected 127.0.0.1')
                else:
                    print('⦿ Test 6 Passed: Event stored correctly in database with correct ip_address')
        else:
            print(f'⍜ Test 6 Failed: Request failed with status code {response.status_code}')
    except Exception as e:
        print(f'⍜ Test 6 Failed: {e}')

def test_ingest_event_with_data(BASE_URL, cursor, session_id):
    event_data_content = {'key1': 'value1', 'key2': 'value2'}
    event_data = {
        'session_id': session_id,
        'event_name': 'test-data-event',
        'data': event_data_content
    }
    try:
        response = requests.post(f'{BASE_URL}/ingest_event', json=event_data)
        if response.status_code == 200:
            print('⦿ Test 7 Passed: Event with data ingested successfully')
            # Check the database
            cursor.execute("SELECT * FROM events WHERE session_id = ? AND event_name = ?", (session_id, 'test-data-event'))
            row = cursor.fetchone()
            if row is None:
                print('⍜ Test 7 Failed: Event not found in database')
            else:
                params = row['params']
                params_dict = json.loads(params)
                if params_dict == event_data_content:
                    print('⦿ Test 7 Passed: Event data stored correctly in database')
                else:
                    print('⍜ Test 7 Failed: Event data not stored correctly in database')
        else:
            print(f'⍜ Test 7 Failed: Request failed with status code {response.status_code}')
    except Exception as e:
        print(f'⍜ Test 7 Failed: {e}')

def main():
    server_process = None

    def kill_server_process():
        nonlocal server_process
        if server_process is not None:
            print("Killing server process...")
            server_process.terminate()
            server_process.wait()

            print()
            print(" === STDOUT ===")
            for line in server_process.stdout:
                print(line)
            print()

            print(" === STDERR ===")
            for line in server_process.stderr:
                print(line)
            print()

            server_process = None

    # Define a signal handler function
    def signal_handler(signum, frame):
        print(f"Received signal {signum}, terminating child process...")
        kill_server_process()

    # Install the signal handler for SIGINT and SIGTERM
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    # Generate a random SECRET_KEY
    SECRET_KEY = generate_secret_key()
    HOST = '127.0.0.1'
    PORT = '8000'
    MAX_JSON_PAYLOAD = '1024'
    db_fd, DB_PATH = tempfile.mkstemp(suffix='.sqlite3')

    env_vars = os.environ.copy()
    env_vars['HOST'] = HOST
    env_vars['PORT'] = PORT
    env_vars['SECRET_KEY'] = SECRET_KEY
    env_vars['MAX_JSON_PAYLOAD'] = MAX_JSON_PAYLOAD
    env_vars['DB_PATH'] = DB_PATH

    # Start the Rust server
    if len(sys.argv) < 2:
        print("Usage: python test_script.py /path/to/rust_server_executable")
        sys.exit(1)

    server_command = sys.argv[1]
    server_process = subprocess.Popen(
        [server_command],
        env=env_vars,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    try:
        print("Starting host application...")
        exit_code = server_process.wait(timeout=2)
        kill_server_process()
        raise AssertionError(f"Unable to start server. Exit code: {exit_code}")    
    except subprocess.TimeoutExpired:
        pass

    try:
        # Wait for the server to be ready
        print("Waiting for connection...")
        wait_for_server(HOST, PORT)

        BASE_URL = f'http://{HOST}:{PORT}'

        # Connect to the database
        conn = sqlite3.connect(DB_PATH)
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        # Run tests
        session_id_1 = test_create_session_without_body(BASE_URL, cursor)
        session_id_2 = test_create_session_with_body(BASE_URL, cursor)
        test_create_session_with_large_body(BASE_URL, MAX_JSON_PAYLOAD)
        test_ingest_event_invalid_session_id(BASE_URL)
        if session_id_2:
            test_ingest_event_missing_event_name(BASE_URL, session_id_2)
            test_ingest_event_with_event_name(BASE_URL, cursor, session_id_2)
            test_ingest_event_with_data(BASE_URL, cursor, session_id_2)
        else:
            print("Skipping some tests due to failure in session creation.")
        session_id_1b = test_create_session_with_user_id(BASE_URL, cursor)

    finally:
        # Clean up
        kill_server_process()
        conn.close()
        os.close(db_fd)
        os.unlink(DB_PATH)

if __name__ == '__main__':
    main()
