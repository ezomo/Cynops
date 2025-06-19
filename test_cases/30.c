/* 8.2 複雑な制御フロー */
// int process_array(int* arr, int size) {
//     int i;
//     int sum = 0;

//     for (i = 0; i < size; i = i + 1) {
//         if (arr[i] < 0) {
//             continue;
//         }

//         switch (arr[i] % 3) {
//             case 0:
//                 sum += arr[i] * 2;
//                 break;
//             case 1:
//                 sum += arr[i];
//                 break;
//             case 2:
//                 sum += arr[i] / 2;
//                 break;
//         }

//         if (sum > 1000) {
//             break;
//         }
//     }

//     return sum;
// }
