document.addEventListener('DOMContentLoaded', () => {
    const todoDescriptionInput = document.getElementById('todo-description');
    const addTodoButton = document.getElementById('add-todo');
    const todoList = document.getElementById('todo-list');

    // API URLs
    const apiUrl = '/api/todos'; // Existing
    const authApiUrl = '/auth';

    // New auth selectors
    const authSection = document.getElementById('auth-section');
    const todoSection = document.getElementById('todo-section');

    const regUsernameInput = document.getElementById('reg-username');
    const regPasswordInput = document.getElementById('reg-password');
    const registerButton = document.getElementById('register-btn');
    const registerMessage = document.getElementById('register-message');

    const loginUsernameInput = document.getElementById('login-username');
    const loginPasswordInput = document.getElementById('login-password');
    const loginButton = document.getElementById('login-btn');
    const loginMessage = document.getElementById('login-message');

    const userInfoDiv = document.getElementById('user-info');
    const loggedInUsernameSpan = document.getElementById('logged-in-username');
    const logoutButton = document.getElementById('logout-btn');

    // --- Authentication Functions ---
    async function handleRegister() {
        const username = regUsernameInput.value.trim();
        const password = regPasswordInput.value.trim();
        if (!username || !password) {
            registerMessage.textContent = 'Username and password are required.';
            return;
        }
        registerMessage.textContent = ''; // Clear previous messages
        try {
            const response = await fetch(`${authApiUrl}/register`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            if (response.ok) {
                const user = await response.json();
                registerMessage.textContent = `User ${user.username} registered successfully! Please login.`;
                regUsernameInput.value = '';
                regPasswordInput.value = '';
            } else {
                const errorText = await response.text(); // Get text first for better error diagnosis
                try {
                    const errorData = JSON.parse(errorText);
                    registerMessage.textContent = `Registration failed: ${response.status} ${errorData.message || errorText}`;
                } catch (e) {
                     registerMessage.textContent = `Registration failed: ${response.status} ${errorText || 'Unknown error'}`;
                }
            }
        } catch (error) {
            console.error('Registration error:', error);
            registerMessage.textContent = 'Registration error: ' + error.message;
        }
    }

    async function handleLogin() {
        const username = loginUsernameInput.value.trim();
        const password = loginPasswordInput.value.trim();
        if (!username || !password) {
            loginMessage.textContent = 'Username and password are required.';
            return;
        }
        loginMessage.textContent = ''; // Clear previous messages
        try {
            const response = await fetch(`${authApiUrl}/login`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            });
            if (response.ok) {
                const data = await response.json();
                localStorage.setItem('session_token', data.session_token);
                localStorage.setItem('user_id', data.user_id); // Store user_id
                localStorage.setItem('username', data.username);
                loginMessage.textContent = 'Login successful!';
                loginUsernameInput.value = '';
                loginPasswordInput.value = '';
                showLoggedInState(data.username);
                fetchTodos();
            } else {
                const errorText = await response.text();
                try {
                    const errorData = JSON.parse(errorText);
                    loginMessage.textContent = `Login failed: ${response.status} ${errorData.message || errorText}`;
                } catch (e) {
                     loginMessage.textContent = `Login failed: ${response.status} ${errorText || 'Unknown error'}`;
                }
                localStorage.removeItem('session_token');
                localStorage.removeItem('username');
                showLoggedOutState();
            }
        } catch (error) {
            console.error('Login error:', error);
            loginMessage.textContent = 'Login error: ' + error.message;
            localStorage.removeItem('session_token');
            localStorage.removeItem('username');
            showLoggedOutState();
        }
    }

    function handleLogout() {
        localStorage.removeItem('session_token');
        localStorage.removeItem('username');
        showLoggedOutState();
        todoList.innerHTML = '<li>Logged out. Please login to see your todos.</li>';
    }

    function showLoggedInState(username) {
        authSection.style.display = 'none';
        todoSection.style.display = 'block';
        userInfoDiv.style.display = 'block';
        loggedInUsernameSpan.textContent = username;
    }

    function showLoggedOutState() {
        authSection.style.display = 'block';
        todoSection.style.display = 'none';
        userInfoDiv.style.display = 'none';
        loggedInUsernameSpan.textContent = '';
    }

    // --- Todo Functions (Modified for Auth) ---
    async function fetchTodos() {
        const token = localStorage.getItem('session_token');
        if (!token) {
            showLoggedOutState();
            todoList.innerHTML = '<li>Please login to see your todos.</li>';
            return;
        }
        try {
            const headers = {};
            if (token) {
                headers['Authorization'] = `Bearer ${token}`;
            }
            const response = await fetch(apiUrl, {
                headers: headers
            });
            if (!response.ok) {
                if (response.status === 401) { // Unauthorized
                    handleLogout(); // Token might be invalid/expired
                    alert("Session expired. Please login again.");
                    return;
                }
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const todos = await response.json();
            renderTodos(todos);
        } catch (error) {
            console.error("Failed to fetch todos:", error);
            todoList.innerHTML = '<li>Failed to load todos. Check console for errors.</li>';
        }
    }

    function renderTodos(todos) {
        todoList.innerHTML = '';
        if (todos.length === 0) {
            todoList.innerHTML = '<li>No todos yet.</li>';
            return;
        }
        todos.forEach(todo => {
            const listItem = document.createElement('li');
            listItem.textContent = todo.description; // Assuming structure is { id, description, completed, user_id, created_at }
            if (todo.completed) {
                listItem.classList.add('completed');
            }

            const completeButton = document.createElement('button');
            completeButton.textContent = 'Complete';
            completeButton.classList.add('complete-btn');
            completeButton.onclick = async () => {
                if (!todo.completed) {
                    await completeTodoItem(todo.id);
                }
            };
            if (todo.completed) {
                completeButton.disabled = true;
                completeButton.textContent = 'Completed';
            }

            listItem.appendChild(completeButton);
            todoList.appendChild(listItem);
        });
    }

    async function addTodoItem() {
        const description = todoDescriptionInput.value.trim();
        if (!description) {
            alert('Please enter a todo description.');
            return;
        }
        const token = localStorage.getItem('session_token');
        if (!token) {
            alert('Please login to add todos.');
            showLoggedOutState();
            return;
        }

        try {
            const headers = {
                'Content-Type': 'application/json' // POST request with JSON body
            };
            if (token) {
                headers['Authorization'] = `Bearer ${token}`;
            }
            const response = await fetch(apiUrl, {
                method: 'POST',
                headers: headers,
                body: JSON.stringify({ description }),
            });
            if (!response.ok) {
                 if (response.status === 401) { // Unauthorized
                    handleLogout();
                    alert("Session expired. Please login again.");
                    return;
                }
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            todoDescriptionInput.value = '';
            fetchTodos();
        } catch (error) {
            console.error("Failed to add todo:", error);
            alert('Failed to add todo.');
        }
    }

    async function completeTodoItem(id) {
        const token = localStorage.getItem('session_token');
        if (!token) {
            alert('Please login to complete todos.');
            showLoggedOutState();
            return;
        }
        try {
            const headers = {}; // PUT request, no JSON body in this case
            if (token) {
                headers['Authorization'] = `Bearer ${token}`;
            }
            const response = await fetch(`${apiUrl}/${id}/complete`, {
                method: 'PUT',
                headers: headers
            });
            if (!response.ok) {
                if (response.status === 401) { // Unauthorized
                    handleLogout();
                    alert("Session expired. Please login again.");
                    return;
                }
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            fetchTodos();
        } catch (error) {
            console.error(`Failed to complete todo ${id}:`, error);
            alert(`Failed to complete todo ${id}.`);
        }
    }

    // Event Listeners
    registerButton.addEventListener('click', handleRegister);
    loginButton.addEventListener('click', handleLogin);
    logoutButton.addEventListener('click', handleLogout);

    addTodoButton.addEventListener('click', addTodoItem);
    todoDescriptionInput.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
            addTodoItem();
        }
    });

    // Initial UI setup
    const currentToken = localStorage.getItem('session_token');
    const currentUsername = localStorage.getItem('username');
    if (currentToken && currentUsername) {
        showLoggedInState(currentUsername);
        fetchTodos();
    } else {
        showLoggedOutState();
    }
});
