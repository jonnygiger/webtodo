document.addEventListener('DOMContentLoaded', () => {
    const todoDescriptionInput = document.getElementById('todo-description');
    const addTodoButton = document.getElementById('add-todo');
    const todoList = document.getElementById('todo-list');

    const apiUrl = '/api/todos';

    async function fetchTodos() {
        try {
            const response = await fetch(apiUrl);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const todos = await response.json();
            renderTodos(todos);
        } catch (error) {
            console.error("Failed to fetch todos:", error);
            todoList.innerHTML = '<li>Failed to load todos.</li>';
        }
    }

    function renderTodos(todos) {
        todoList.innerHTML = ''; // Clear existing todos
        if (todos.length === 0) {
            todoList.innerHTML = '<li>No todos yet.</li>';
            return;
        }
        todos.forEach(todo => {
            const listItem = document.createElement('li');
            listItem.textContent = todo.description;
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

        try {
            const response = await fetch(apiUrl, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ description }),
            });
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            // const newTodo = await response.json(); // Not strictly needed to re-render from this
            todoDescriptionInput.value = ''; // Clear input
            fetchTodos(); // Refresh the list
        } catch (error) {
            console.error("Failed to add todo:", error);
            alert('Failed to add todo.');
        }
    }

    async function completeTodoItem(id) {
        try {
            const response = await fetch(`${apiUrl}/${id}/complete`, {
                method: 'PUT',
            });
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            fetchTodos(); // Refresh the list
        } catch (error) {
            console.error(`Failed to complete todo ${id}:`, error);
            alert(`Failed to complete todo ${id}.`);
        }
    }

    addTodoButton.addEventListener('click', addTodoItem);
    todoDescriptionInput.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
            addTodoItem();
        }
    });

    // Initial fetch of todos
    fetchTodos();
});
